# File Data Sources

File data sources allow you to index off-chain data stored on IPFS or Arweave. They're triggered when a contract emits an event containing a content hash.

## When to Use

- **NFT metadata** — Token URIs pointing to IPFS
- **On-chain content** — Documents, images, JSON stored on IPFS/Arweave
- **Decentralized storage** — Any CID-referenced content

## Defining File Data Sources

In `subgraph.yaml`:

```yaml
templates:
  - kind: file/ipfs
    name: TokenMetadata
    network: mainnet
    mapping:
      kind: ethereum/events
      apiVersion: 0.0.7
      language: wasm/assemblyscript
      entities:
        - TokenMetadata
      abis: []
      file: ./build/subgraph.wasm
      handler: handleTokenMetadata
```

For Arweave:

```yaml
templates:
  - kind: file/arweave
    name: ArweaveContent
    # ...
```

## Triggering File Data Sources

When you detect a content hash in an event, create the file data source:

```rust
use crate::generated::templates::TokenMetadata;

#[handler]
fn handle_transfer(event: TransferEvent) {
    let token_id = &event.params.token_id;

    // Get the token URI from the contract
    if let Some(uri) = get_token_uri(token_id) {
        // Extract IPFS hash from URI
        if let Some(ipfs_hash) = extract_ipfs_hash(&uri) {
            // Create file data source
            TokenMetadata::create(ipfs_hash);
        }
    }
}

fn extract_ipfs_hash(uri: &str) -> Option<String> {
    if uri.starts_with("ipfs://") {
        Some(uri.strip_prefix("ipfs://")?.to_string())
    } else if uri.contains("/ipfs/") {
        uri.split("/ipfs/").nth(1).map(|s| s.to_string())
    } else {
        None
    }
}
```

## File Handler

The file handler receives the raw file content:

```rust
use yogurt_runtime::json;

#[handler]
fn handle_token_metadata(content: Bytes) {
    // Parse JSON content
    let json = match json::from_bytes(&content) {
        Ok(j) => j,
        Err(_) => return,  // Invalid JSON, skip
    };

    // Extract the CID from the data source
    let cid = data_source::string_param();

    // Parse metadata fields
    let name = json.get("name")
        .and_then(|v| v.to_string())
        .unwrap_or_default();

    let description = json.get("description")
        .and_then(|v| v.to_string())
        .unwrap_or_default();

    let image = json.get("image")
        .and_then(|v| v.to_string())
        .unwrap_or_default();

    // Save metadata entity
    TokenMetadata::builder(cid)
        .name(&name)
        .description(&description)
        .image(&image)
        .save();
}
```

## JSON Parsing

yogurt provides JSON parsing utilities:

```rust
use yogurt_runtime::json::{self, JsonValue};

fn parse_metadata(content: &Bytes) -> Option<Metadata> {
    let json = json::from_bytes(content).ok()?;

    // Access nested values
    let attributes = json.get("attributes")?;

    if let JsonValue::Array(attrs) = attributes {
        for attr in attrs {
            let trait_type = attr.get("trait_type")?.to_string()?;
            let value = attr.get("value")?.to_string()?;
            // Process attribute...
        }
    }

    Some(Metadata { /* ... */ })
}
```

## Testing File Data Sources

Use `mock_ipfs_cat` to test file handlers:

```rust
use yogurt_runtime::testing::*;

#[test]
fn test_metadata_parsing() {
    clear_store();

    let metadata = r#"{
        "name": "Cool NFT #1",
        "description": "A very cool NFT",
        "image": "ipfs://QmXyz..."
    }"#;

    mock_ipfs_cat("QmAbc123", metadata.as_bytes());

    // Trigger your handler logic
    handle_token_metadata(Bytes::from(metadata.as_bytes()));

    assert_entity_exists::<TokenMetadata>("QmAbc123");
}
```

## Error Handling

File data sources may fail if content is unavailable. Handle gracefully:

```rust
#[handler]
fn handle_token_metadata(content: Bytes) {
    // Early return on parse failure
    let json = match json::from_bytes(&content) {
        Ok(j) => j,
        Err(e) => {
            log::warning!("Failed to parse metadata: {}", e);
            return;
        }
    };

    // Continue processing...
}
```
