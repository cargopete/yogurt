#!/usr/bin/env bash
#
# Test yogurt deployment against a local graph-node.
#
# Usage:
#   ./scripts/test-deploy.sh          # Full test (start infra, build, deploy, verify)
#   ./scripts/test-deploy.sh --up     # Just start infrastructure
#   ./scripts/test-deploy.sh --down   # Tear down infrastructure
#   ./scripts/test-deploy.sh --deploy # Just deploy (assumes infra is running)
#

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
TEST_SUBGRAPH="$PROJECT_ROOT/tests/integration/erc20-transfer"
SUBGRAPH_NAME="test/erc20-transfer"

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠${NC} $1"
}

log_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗${NC} $1"
}

log_step() {
    echo -e "\n${CYAN}━━━ $1 ━━━${NC}\n"
}

# Check required tools
check_requirements() {
    log_step "Checking requirements"

    local missing=()

    command -v docker >/dev/null 2>&1 || missing+=("docker")
    command -v docker-compose >/dev/null 2>&1 || command -v "docker compose" >/dev/null 2>&1 || missing+=("docker-compose")
    command -v cargo >/dev/null 2>&1 || missing+=("cargo")
    command -v curl >/dev/null 2>&1 || missing+=("curl")

    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing[*]}"
        exit 1
    fi

    log_success "All requirements met"
}

# Docker compose wrapper (handles both v1 and v2)
dc() {
    if command -v docker-compose >/dev/null 2>&1; then
        docker-compose -f "$COMPOSE_FILE" "$@"
    else
        docker compose -f "$COMPOSE_FILE" "$@"
    fi
}

# Start infrastructure
start_infra() {
    log_step "Starting infrastructure"

    log "Pulling latest images..."
    dc pull --quiet

    log "Starting containers..."
    dc up -d

    log "Waiting for services to be healthy..."

    # Wait for IPFS (use Docker health check)
    local retries=30
    while true; do
        local ipfs_health=$(docker inspect --format='{{.State.Health.Status}}' scripts-ipfs-1 2>/dev/null || echo "unknown")
        if [ "$ipfs_health" = "healthy" ]; then
            break
        fi
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            log_error "IPFS failed to start"
            dc logs ipfs
            exit 1
        fi
        sleep 1
    done
    log_success "IPFS is ready"

    # Wait for graph-node (JSON-RPC, needs POST)
    retries=60
    while ! curl -sf -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"subgraph_list","params":{},"id":1}' \
        http://localhost:8020 >/dev/null 2>&1; do
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            log_error "Graph-node failed to start"
            dc logs graph-node
            exit 1
        fi
        sleep 2
    done
    log_success "Graph-node is ready"

    log_success "All services running"
}

# Stop infrastructure
stop_infra() {
    log_step "Stopping infrastructure"
    dc down -v
    log_success "Infrastructure stopped"
}

# Build the test subgraph
build_subgraph() {
    log_step "Building test subgraph"

    cd "$TEST_SUBGRAPH"

    log "Running cargo build..."
    cargo build --target wasm32-unknown-unknown --release 2>&1 | while read -r line; do
        echo "  $line"
    done

    # Copy to build directory
    mkdir -p build
    cp target/wasm32-unknown-unknown/release/*.wasm build/subgraph.wasm 2>/dev/null || \
    cp "$PROJECT_ROOT/target/wasm32-unknown-unknown/release/erc20_transfer.wasm" build/subgraph.wasm

    local size=$(wc -c < build/subgraph.wasm | tr -d ' ')
    log_success "Built WASM: $(echo "scale=1; $size / 1024" | bc) KB"

    cd "$PROJECT_ROOT"
}

# Build yogurt CLI
build_cli() {
    log_step "Building yogurt CLI"

    cd "$PROJECT_ROOT"
    cargo build -p yogurt-cli --release 2>&1 | while read -r line; do
        echo "  $line"
    done

    log_success "CLI built"
}

# Deploy subgraph
deploy_subgraph() {
    log_step "Deploying subgraph"

    cd "$TEST_SUBGRAPH"

    log "Deploying $SUBGRAPH_NAME..."
    "$PROJECT_ROOT/target/release/yogurt" deploy "$SUBGRAPH_NAME" \
        --node http://localhost:8020 \
        --ipfs http://localhost:5001 \
        --version v1.0.0 2>&1 | while read -r line; do
        echo "  $line"
    done

    if [ $? -eq 0 ]; then
        log_success "Deployment successful"
    else
        log_error "Deployment failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Verify deployment
verify_deployment() {
    log_step "Verifying deployment"

    log "Checking subgraph status..."

    # Wait a moment for indexing to start
    sleep 3

    # Query the indexing status
    local query='{"query":"{ indexingStatuses { subgraph health synced chains { network latestBlock { number } } } }"}'
    local response
    response=$(curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d "$query" \
        http://localhost:8030/graphql)

    if [ $? -eq 0 ]; then
        log "Indexing status:"
        echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
        log_success "Subgraph is deployed and indexing"
    else
        log_warn "Could not query indexing status (this may be normal initially)"
    fi

    log "GraphQL endpoint: http://localhost:8000/subgraphs/name/$SUBGRAPH_NAME"
}

# Show logs
show_logs() {
    log_step "Recent graph-node logs"
    dc logs --tail=50 graph-node
}

# Main
main() {
    case "${1:-}" in
        --up)
            check_requirements
            start_infra
            ;;
        --down)
            stop_infra
            ;;
        --deploy)
            build_subgraph
            deploy_subgraph
            verify_deployment
            ;;
        --logs)
            show_logs
            ;;
        --help|-h)
            echo "Usage: $0 [--up|--down|--deploy|--logs|--help]"
            echo ""
            echo "  (no args)  Full test: start infra, build, deploy, verify"
            echo "  --up       Start infrastructure only"
            echo "  --down     Stop infrastructure"
            echo "  --deploy   Build and deploy (assumes infra is running)"
            echo "  --logs     Show graph-node logs"
            echo "  --help     Show this help"
            ;;
        *)
            check_requirements
            start_infra
            build_cli
            build_subgraph
            deploy_subgraph
            verify_deployment
            echo ""
            log_success "All done! Query your subgraph at:"
            echo "  http://localhost:8000/subgraphs/name/$SUBGRAPH_NAME"
            ;;
    esac
}

main "$@"
