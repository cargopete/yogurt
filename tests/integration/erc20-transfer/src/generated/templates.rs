//! Auto-generated data source templates — do not edit

use alloc::string::ToString;
use yogurt_runtime::data_source;

/// Data source template: `TokenMetadata`
///
/// Kind: `file/ipfs`
pub struct TokenMetadata;

impl TokenMetadata {
    /// Create a new file data source for the given content identifier.
///
/// For IPFS: pass the CID (e.g., "QmXxx...")
/// For Arweave: pass the transaction ID
pub fn create(content_id: &str) {
data_source::create("TokenMetadata", &[content_id.to_string()]);
}
}

