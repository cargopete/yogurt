# Subgraph Studio

Deploy to The Graph's hosted Subgraph Studio for production use.

## Prerequisites

1. **Create account** — Sign up at [thegraph.com/studio](https://thegraph.com/studio/)
2. **Create subgraph** — Use the Studio web UI to create a new subgraph
3. **Get deploy key** — Copy your deploy key from the Studio dashboard

## Authentication

Store your deploy key locally:

```bash
yogurt auth <your-deploy-key>
```

The key is saved to `~/.config/yogurt/auth.json` (or equivalent on your OS).

## Deploy

```bash
yogurt deploy <subgraph-slug> --studio --version <version>
```

Example:

```bash
yogurt deploy my-cool-subgraph --studio --version 0.0.1
```

The subgraph slug must match what you created in Studio.

## Version Labeling

Every deployment requires a version label:

```bash
yogurt deploy my-subgraph --studio --version 0.0.1
yogurt deploy my-subgraph --studio --version 0.0.2
yogurt deploy my-subgraph --studio --version 1.0.0
```

Use semantic versioning for clarity.

## Deployment Process

1. **Build** — Compile to WASM (if not already built)
2. **Upload** — Send files to Studio's IPFS
3. **Deploy** — Create new version in Studio

## After Deployment

### Check Status

View indexing progress in the Studio dashboard:
- Synced percentage
- Latest block indexed
- Any errors

### Query Endpoint

Studio provides a temporary query endpoint:
```
https://api.studio.thegraph.com/query/<id>/<subgraph>/<version>
```

### Publish to Network

To publish to the decentralized network:
1. Go to Studio dashboard
2. Click "Publish"
3. Choose curation amount
4. Confirm transaction

Published subgraphs get a permanent query endpoint and can earn indexing rewards.

## Rate Limits

Studio has rate limits for the free tier:
- 100,000 queries/month
- 1000 queries/day during development

For production traffic, publish to the decentralized network.

## Multiple Environments

### Development

```bash
yogurt deploy my-subgraph-dev --studio --version 0.0.1-dev
```

### Staging

```bash
yogurt deploy my-subgraph-staging --studio --version 0.0.1-rc1
```

### Production

```bash
yogurt deploy my-subgraph --studio --version 1.0.0
```

## CI/CD Deployment

Example GitHub Actions workflow:

```yaml
name: Deploy to Studio

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install yogurt
        run: cargo install --path crates/yogurt-cli

      - name: Build
        run: |
          yogurt codegen
          yogurt build --release

      - name: Deploy
        env:
          GRAPH_AUTH_TOKEN: ${{ secrets.GRAPH_AUTH_TOKEN }}
        run: |
          yogurt auth $GRAPH_AUTH_TOKEN
          yogurt deploy my-subgraph --studio --version ${{ github.ref_name }}
```

Store your deploy key as a GitHub secret (`GRAPH_AUTH_TOKEN`).

## Troubleshooting

### Authentication Failed

```
Error: authentication failed
```

Re-authenticate:
```bash
yogurt auth <your-deploy-key>
```

### Subgraph Not Found

```
Error: subgraph not found
```

Make sure:
1. The subgraph exists in Studio
2. The slug matches exactly (case-sensitive)
3. You're authenticated with the correct account

### Version Already Exists

```
Error: version already exists
```

Use a new version number:
```bash
yogurt deploy my-subgraph --studio --version 0.0.2
```

### Build Failed in Studio

Check the Studio dashboard for detailed error messages. Common issues:
- Invalid manifest
- Missing ABIs
- Handler compilation errors

## Costs

- **Studio (development)**: Free with rate limits
- **Decentralized network**: Pay per query + curation deposit

See [The Graph pricing](https://thegraph.com/docs/en/billing/) for details.
