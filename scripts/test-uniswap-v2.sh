#!/usr/bin/env bash
#
# Test Uniswap V2 subgraph deployment against a local graph-node.
#
# Usage:
#   ./scripts/test-uniswap-v2.sh          # Full test (start infra, build, deploy, verify)
#   ./scripts/test-uniswap-v2.sh --up     # Just start infrastructure
#   ./scripts/test-uniswap-v2.sh --down   # Tear down infrastructure
#   ./scripts/test-uniswap-v2.sh --deploy # Just deploy (assumes infra is running)
#   ./scripts/test-uniswap-v2.sh --logs   # Show graph-node logs
#   ./scripts/test-uniswap-v2.sh --status # Check indexing status
#

set -euo pipefail

# Colours for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
DIM='\033[2m'
NC='\033[0m' # No colour

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
TEST_SUBGRAPH="$PROJECT_ROOT/tests/integration/uniswap-v2"
SUBGRAPH_NAME="test/uniswap-v2"
YOGURT_CLI="$PROJECT_ROOT/target/release/yogurt"

# Endpoints
IPFS_API="http://localhost:5001"
GRAPH_NODE_ADMIN="http://localhost:8020"
GRAPH_NODE_INDEX="http://localhost:8030"
GRAPH_NODE_QUERY="http://localhost:8000"

log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

log_debug() {
    echo -e "${DIM}[$(date '+%H:%M:%S')] $1${NC}"
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
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# Check required tools
check_requirements() {
    log_step "Checking requirements"

    local missing=()

    log_debug "Checking for docker..."
    if command -v docker >/dev/null 2>&1; then
        log_debug "  docker: $(docker --version)"
    else
        missing+=("docker")
    fi

    log_debug "Checking for docker-compose..."
    if command -v docker-compose >/dev/null 2>&1; then
        log_debug "  docker-compose: $(docker-compose --version)"
    elif docker compose version >/dev/null 2>&1; then
        log_debug "  docker compose: $(docker compose version)"
    else
        missing+=("docker-compose")
    fi

    log_debug "Checking for cargo..."
    if command -v cargo >/dev/null 2>&1; then
        log_debug "  cargo: $(cargo --version)"
    else
        missing+=("cargo")
    fi

    log_debug "Checking for curl..."
    if command -v curl >/dev/null 2>&1; then
        log_debug "  curl: $(curl --version | head -1)"
    else
        missing+=("curl")
    fi

    log_debug "Checking for wasm32 target..."
    if rustup target list --installed | grep -q wasm32-unknown-unknown; then
        log_debug "  wasm32-unknown-unknown: installed"
    else
        log_warn "wasm32-unknown-unknown target not installed. Installing..."
        rustup target add wasm32-unknown-unknown
    fi

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

    log "Checking for existing containers..."
    local running=$(dc ps -q 2>/dev/null | wc -l | tr -d ' ')
    if [ "$running" -gt 0 ]; then
        log_warn "Containers already running. Use --down first to reset."
        log_debug "Running containers:"
        dc ps
    fi

    log "Pulling latest images..."
    dc pull 2>&1 | while read -r line; do
        log_debug "  $line"
    done

    log "Starting containers..."
    dc up -d 2>&1 | while read -r line; do
        log_debug "  $line"
    done

    log "Container status:"
    dc ps

    log "Waiting for IPFS to be ready..."
    local retries=30
    while true; do
        local ipfs_health=$(docker inspect --format='{{.State.Health.Status}}' scripts-ipfs-1 2>/dev/null || echo "unknown")
        if [ "$ipfs_health" = "healthy" ]; then
            break
        fi
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            log_error "IPFS failed to become healthy after 30 seconds"
            log "IPFS logs:"
            dc logs --tail=30 ipfs
            exit 1
        fi
        log_debug "  Waiting for IPFS... (status: $ipfs_health, $retries attempts left)"
        sleep 1
    done
    log_success "IPFS is ready"

    log "Waiting for Postgres to be ready..."
    retries=30
    while ! dc exec -T postgres pg_isready -U graph-node >/dev/null 2>&1; do
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            log_error "Postgres failed to start after 30 seconds"
            log "Postgres logs:"
            dc logs --tail=30 postgres
            exit 1
        fi
        log_debug "  Waiting for Postgres... ($retries attempts left)"
        sleep 1
    done
    log_success "Postgres is ready"

    log "Waiting for graph-node to be ready..."
    retries=60
    # graph-node admin is JSON-RPC, so we need to POST a valid request
    while ! curl -sf -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"subgraph_list","params":{},"id":1}' \
        "$GRAPH_NODE_ADMIN" >/dev/null 2>&1; do
        retries=$((retries - 1))
        if [ $retries -le 0 ]; then
            log_error "Graph-node failed to start after 120 seconds"
            log "Graph-node logs:"
            dc logs --tail=50 graph-node
            exit 1
        fi
        log_debug "  Waiting for graph-node... ($retries attempts left)"
        sleep 2
    done
    log_success "Graph-node is ready"

    # Show Ethereum RPC being used
    log "Checking Ethereum connection..."
    local eth_rpc="${ETHEREUM_RPC:-https://eth.llamarpc.com}"
    log_debug "  RPC endpoint: $eth_rpc"

    # Try to get latest block
    local block_response=$(curl -sf -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "$eth_rpc" 2>/dev/null || echo '{"error":"failed"}')

    if echo "$block_response" | grep -q '"result"'; then
        local block_hex=$(echo "$block_response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        local block_num=$((block_hex))
        log_success "Ethereum RPC connected (latest block: $block_num)"
    else
        log_warn "Could not verify Ethereum RPC connection"
        log_debug "  Response: $block_response"
    fi

    echo ""
    log_success "All services running"
    log "  IPFS API:        $IPFS_API"
    log "  Graph Admin:     $GRAPH_NODE_ADMIN"
    log "  Graph Query:     $GRAPH_NODE_QUERY"
    log "  Graph Index:     $GRAPH_NODE_INDEX"
}

# Stop infrastructure
stop_infra() {
    log_step "Stopping infrastructure"

    log "Stopping and removing containers..."
    dc down -v 2>&1 | while read -r line; do
        log_debug "  $line"
    done

    log_success "Infrastructure stopped"
}

# Build yogurt CLI
build_cli() {
    log_step "Building yogurt CLI"

    if [ -f "$YOGURT_CLI" ]; then
        local cli_age=$(($(date +%s) - $(stat -f %m "$YOGURT_CLI" 2>/dev/null || stat -c %Y "$YOGURT_CLI" 2>/dev/null || echo 0)))
        if [ $cli_age -lt 300 ]; then
            log "CLI binary is recent (${cli_age}s old), skipping rebuild"
            log_success "Using existing CLI: $YOGURT_CLI"
            return 0
        fi
    fi

    cd "$PROJECT_ROOT"
    log "Running: cargo build -p yogurt-cli --release"

    if cargo build -p yogurt-cli --release 2>&1 | while read -r line; do
        # Only show interesting lines
        if echo "$line" | grep -qE "Compiling|Finished|error|warning:.*yogurt"; then
            log_debug "  $line"
        fi
    done; then
        log_success "CLI built: $YOGURT_CLI"
    else
        log_error "CLI build failed"
        exit 1
    fi
}

# Build the test subgraph
build_subgraph() {
    log_step "Building Uniswap V2 subgraph"

    cd "$TEST_SUBGRAPH"
    log "Working directory: $TEST_SUBGRAPH"

    # Run codegen first
    log "Running codegen..."
    if "$YOGURT_CLI" codegen 2>&1 | while read -r line; do
        log_debug "  $line"
    done; then
        log_success "Codegen complete"
    else
        log_error "Codegen failed"
        exit 1
    fi

    # Build WASM using yogurt build (handles copying to build/subgraph.wasm)
    log "Running: yogurt build --release"
    if "$YOGURT_CLI" build --release 2>&1 | while read -r line; do
        log_debug "  $line"
    done; then
        log_success "WASM build complete"
    else
        log_error "WASM build failed"
        exit 1
    fi

    # Check output exists
    if [ ! -f "build/subgraph.wasm" ]; then
        log_error "WASM file not found: build/subgraph.wasm"
        exit 1
    fi

    local size=$(wc -c < "build/subgraph.wasm" | tr -d ' ')
    local size_kb=$(echo "scale=1; $size / 1024" | bc)
    log_success "WASM size: ${size_kb} KB (build/subgraph.wasm)"

    # Validate
    log "Validating WASM exports..."
    if "$YOGURT_CLI" validate build/subgraph.wasm 2>&1 | while read -r line; do
        log_debug "  $line"
    done; then
        log_success "WASM validation passed"
    else
        log_error "WASM validation failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Deploy subgraph
deploy_subgraph() {
    log_step "Deploying Uniswap V2 subgraph"

    cd "$TEST_SUBGRAPH"

    # Check if subgraph already exists and remove it
    log "Checking for existing deployment..."
    local existing=$(curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"subgraph_list","params":{},"id":1}' \
        "$GRAPH_NODE_ADMIN" 2>/dev/null || echo '{}')

    if echo "$existing" | grep -q "$SUBGRAPH_NAME"; then
        log_warn "Subgraph already exists, removing..."
        curl -sf -X POST \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"subgraph_remove\",\"params\":{\"name\":\"$SUBGRAPH_NAME\"},\"id\":1}" \
            "$GRAPH_NODE_ADMIN" >/dev/null 2>&1 || true
        sleep 2
    fi

    log "Deploying: $SUBGRAPH_NAME"
    log_debug "  Node: $GRAPH_NODE_ADMIN"
    log_debug "  IPFS: $IPFS_API"
    log_debug "  Manifest: $TEST_SUBGRAPH/subgraph.yaml"

    if "$YOGURT_CLI" deploy "$SUBGRAPH_NAME" \
        --node "$GRAPH_NODE_ADMIN" \
        --ipfs "$IPFS_API" \
        --version v1.0.0 2>&1 | while read -r line; do
        log_debug "  $line"
    done; then
        log_success "Deployment command completed"
    else
        log_error "Deployment failed"
        log "Checking graph-node logs for errors..."
        dc logs --tail=30 graph-node | grep -iE "error|failed|panic" || true
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Check indexing status
check_status() {
    log_step "Checking indexing status"

    log "Querying indexing status..."

    local query='{"query":"{ indexingStatuses { subgraph synced health fatalError { message } chains { network latestBlock { number hash } chainHeadBlock { number } } } }"}'

    local response=$(curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d "$query" \
        "$GRAPH_NODE_INDEX/graphql" 2>/dev/null || echo '{"error":"connection failed"}')

    if echo "$response" | grep -q '"error"'; then
        log_warn "Could not query indexing status"
        log_debug "Response: $response"
        return 1
    fi

    # Pretty print if python is available
    if command -v python3 >/dev/null 2>&1; then
        echo "$response" | python3 -c "
import sys, json
data = json.load(sys.stdin)
statuses = data.get('data', {}).get('indexingStatuses', [])
for s in statuses:
    print(f\"  Subgraph: {s.get('subgraph', 'unknown')[:12]}...\")
    print(f\"    Health: {s.get('health', 'unknown')}\")
    print(f\"    Synced: {s.get('synced', 'unknown')}\")
    if s.get('fatalError'):
        print(f\"    Error:  {s['fatalError'].get('message', 'unknown')}\")
    for chain in s.get('chains', []):
        latest = chain.get('latestBlock', {}).get('number', '?')
        head = chain.get('chainHeadBlock', {}).get('number', '?')
        print(f\"    Chain:  {chain.get('network', 'unknown')} (indexed: {latest}, head: {head})\")
" 2>/dev/null || echo "$response"
    else
        echo "$response"
    fi

    # Check for fatal errors
    if echo "$response" | grep -q '"fatalError"'; then
        if echo "$response" | grep -q '"fatalError":null'; then
            log_success "No fatal errors"
        else
            log_error "Subgraph has a fatal error!"
            log "Check logs with: ./scripts/test-uniswap-v2.sh --logs"
        fi
    fi
}

# Verify deployment and show helpful info
verify_deployment() {
    log_step "Verifying deployment"

    # Wait a moment for indexing to start
    log "Waiting for indexing to begin..."
    sleep 5

    check_status

    echo ""
    log "Uniswap V2 Factory deployment info:"
    log "  Contract: 0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
    log "  Start block: 10,000,835 (May 2020)"
    log "  Note: Full sync will take time. Check --status periodically."
    echo ""
    log "Endpoints:"
    log "  GraphQL: $GRAPH_NODE_QUERY/subgraphs/name/$SUBGRAPH_NAME"
    log "  Index:   $GRAPH_NODE_INDEX/graphql"
    echo ""
    log "Example query:"
    cat << 'EOF'
  curl -X POST \
    -H "Content-Type: application/json" \
    -d '{"query":"{ factories(first:1) { id pairCount } }"}' \
    http://localhost:8000/subgraphs/name/test/uniswap-v2
EOF
}

# Show logs
show_logs() {
    log_step "Graph-node logs (last 100 lines)"
    dc logs --tail=100 graph-node
}

# Main
main() {
    echo -e "${CYAN}"
    echo "  ╔═══════════════════════════════════════════════════════════╗"
    echo "  ║           Uniswap V2 Subgraph Test Script                 ║"
    echo "  ║                    yogurt toolchain                       ║"
    echo "  ╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    case "${1:-}" in
        --up)
            check_requirements
            start_infra
            ;;
        --down)
            stop_infra
            ;;
        --deploy)
            build_cli
            build_subgraph
            deploy_subgraph
            verify_deployment
            ;;
        --logs)
            show_logs
            ;;
        --status)
            check_status
            ;;
        --help|-h)
            echo "Usage: $0 [--up|--down|--deploy|--logs|--status|--help]"
            echo ""
            echo "  (no args)  Full test: start infra, build, deploy, verify"
            echo "  --up       Start infrastructure only (IPFS, Postgres, graph-node)"
            echo "  --down     Stop infrastructure and remove volumes"
            echo "  --deploy   Build and deploy (assumes infra is running)"
            echo "  --logs     Show recent graph-node logs"
            echo "  --status   Check subgraph indexing status"
            echo "  --help     Show this help"
            echo ""
            echo "Environment variables:"
            echo "  ETHEREUM_RPC   Mainnet RPC URL (default: https://eth.llamarpc.com)"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Full test with default RPC"
            echo "  ETHEREUM_RPC=https://... $0           # Use custom RPC"
            echo "  $0 --up && $0 --deploy                # Start infra, then deploy"
            ;;
        *)
            check_requirements
            start_infra
            build_cli
            build_subgraph
            deploy_subgraph
            verify_deployment
            echo ""
            log_success "All done!"
            log "Monitor indexing: ./scripts/test-uniswap-v2.sh --status"
            log "View logs:        ./scripts/test-uniswap-v2.sh --logs"
            log "Tear down:        ./scripts/test-uniswap-v2.sh --down"
            ;;
    esac
}

main "$@"
