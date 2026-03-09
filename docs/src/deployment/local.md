# Local Graph Node

Deploy to a local graph-node for development and testing.

## Prerequisites

### Build Tools

You need **binaryen** (provides `wasm-opt`) for graph-node compatible WASM:

```bash
# macOS
brew install binaryen

# Ubuntu/Debian
apt-get install binaryen

# Or download from https://github.com/WebAssembly/binaryen/releases
```

The `yogurt build` command uses wasm-opt to convert modern WASM features (bulk memory operations) to MVP-compatible code that graph-node supports.

### Graph Node

You need a running graph-node with:
- **IPFS** — For storing subgraph files
- **PostgreSQL** — For storing indexed data
- **Ethereum RPC** — For reading blockchain data

## Quick Setup with Docker

The easiest way to run graph-node locally:

```bash
# Clone graph-node
git clone https://github.com/graphprotocol/graph-node
cd graph-node/docker

# Configure Ethereum RPC
echo 'ethereum=mainnet:http://host.docker.internal:8545' > .env
# Or use a remote RPC:
echo 'ethereum=mainnet:https://mainnet.infura.io/v3/YOUR_KEY' > .env

# Start services
docker-compose up -d
```

Default ports:
- **8000** — GraphQL HTTP
- **8001** — GraphQL WebSocket
- **8020** — JSON-RPC (for deployment)
- **8030** — Index node status
- **5001** — IPFS API

## Deploy Command

```bash
yogurt deploy <name> [options]
```

### Basic Deployment

```bash
yogurt deploy myaccount/my-subgraph
```

Uses default endpoints:
- Node: `http://localhost:8020`
- IPFS: `http://localhost:5001`

### Custom Endpoints

```bash
yogurt deploy myaccount/my-subgraph \
  --node http://localhost:8020 \
  --ipfs http://localhost:5001
```

### Version Label

```bash
yogurt deploy myaccount/my-subgraph --version v1.0.0
```

The version label is for your reference; graph-node tracks versions internally.

## Deployment Process

1. **Upload to IPFS** — Schema, manifest, WASM, and ABIs
2. **Create subgraph** — Register name with graph-node (first deploy only)
3. **Deploy version** — Point subgraph to new IPFS hash

## Checking Status

### GraphQL Playground

Open http://localhost:8000/subgraphs/name/myaccount/my-subgraph

### Indexing Status

```bash
curl -s http://localhost:8030/graphql -H 'Content-Type: application/json' \
  -d '{"query": "{ indexingStatuses { subgraph synced health fatalError { message } } }"}' \
  | jq .
```

### Graph Node Logs

```bash
docker-compose logs -f graph-node
```

## Redeploying

Just run deploy again:

```bash
yogurt deploy myaccount/my-subgraph
```

Graph-node creates a new version and starts indexing from the start block.

## Removing a Subgraph

```bash
curl -X POST http://localhost:8020 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"subgraph_remove","params":{"name":"myaccount/my-subgraph"},"id":1}'
```

## Troubleshooting

### Connection Refused

```
Error: connection refused
```

Check that graph-node is running:
```bash
docker-compose ps
```

### IPFS Upload Failed

```
Error: failed to upload to IPFS
```

Check IPFS is accessible:
```bash
curl http://localhost:5001/api/v0/id
```

### Subgraph Failed

Check the indexing status and logs:
```bash
docker-compose logs graph-node | grep -i error
```

Common issues:
- Invalid start block
- RPC errors (rate limiting, wrong network)
- Handler panics

### Handler Errors

View detailed error in logs:
```bash
docker-compose logs -f graph-node 2>&1 | grep -A 5 "Handler error"
```

## Using Scripts

The repository includes helper scripts:

```bash
# Start infrastructure
./scripts/test-deploy.sh --up

# Deploy
./scripts/test-deploy.sh --deploy

# View logs
./scripts/test-deploy.sh --logs

# Stop
./scripts/test-deploy.sh --down
```

## Custom Ethereum RPC

```bash
ETHEREUM_RPC="https://mainnet.infura.io/v3/YOUR_KEY" ./scripts/test-deploy.sh --up
```

Or edit `docker/docker-compose.yml` directly.
