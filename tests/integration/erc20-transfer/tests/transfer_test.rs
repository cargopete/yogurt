//! Tests for the Transfer handler.
//!
//! These tests run natively (no WASM) using yogurt's testing framework.

extern crate alloc;

use alloc::vec::Vec;
use yogurt_runtime::prelude::*;
use yogurt_runtime::testing::*;
use yogurt_runtime::types::Entity;

// Import the generated types and handlers
use erc20_transfer::generated::{Transfer, TransferCall, TransferEvent, TransferInputs, TransferOutputs, TransferParams};
use erc20_transfer::mappings::{handle_transfer, handle_transfer_call};

#[test]
fn test_handle_transfer_creates_entity() {
    // Clear the mock store before each test
    clear_store();

    // Create a test event using EventBuilder
    let event: TransferEvent = EventBuilder::new()
        .address(Address::from([0xABu8; 20]))
        .block_number(12345678)
        .block_timestamp(1700000000)
        .transaction_hash([0xDEu8; 32])
        .log_index(42)
        .params(TransferParams {
            from: Address::from([0x11u8; 20]),
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(1_000_000_000_000_000_000), // 1 ETH in wei
        })
        .build();

    // Call the handler
    handle_transfer(event);

    // Verify that a Transfer entity was created
    // The ID format is "{tx_hash}-{log_index}"
    let expected_id = "0xdededededededededededededededededededededededededededededededede-42";
    assert_entity_exists::<Transfer>(expected_id);

    // Load the entity and verify its fields
    let transfer = Transfer::load(expected_id).expect("Transfer should exist");
    assert_eq!(transfer.value().to_string(), "1000000000000000000");
    assert_eq!(transfer.block_number().to_string(), "12345678");
}

#[test]
fn test_handle_transfer_with_different_values() {
    clear_store();

    // Test with a smaller transfer amount
    let event: TransferEvent = EventBuilder::new()
        .block_number(100)
        .log_index(0)
        .transaction_hash([0x01u8; 32])
        .params(TransferParams {
            from: Address::from([0xAAu8; 20]),
            to: Address::from([0xBBu8; 20]),
            value: BigInt::from_u64(42),
        })
        .build();

    handle_transfer(event);

    let expected_id = "0x0101010101010101010101010101010101010101010101010101010101010101-0";
    let transfer = Transfer::load(expected_id).expect("Transfer should exist");
    assert_eq!(transfer.value().to_string(), "42");
}

#[test]
fn test_multiple_transfers() {
    clear_store();

    // Create first transfer
    let event1: TransferEvent = EventBuilder::new()
        .transaction_hash([0x01u8; 32])
        .log_index(0)
        .params(TransferParams {
            from: Address::from([0x11u8; 20]),
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(100),
        })
        .build();

    // Create second transfer
    let event2: TransferEvent = EventBuilder::new()
        .transaction_hash([0x01u8; 32])
        .log_index(1)
        .params(TransferParams {
            from: Address::from([0x22u8; 20]),
            to: Address::from([0x33u8; 20]),
            value: BigInt::from_u64(50),
        })
        .build();

    handle_transfer(event1);
    handle_transfer(event2);

    // Both entities should exist
    assert_eq!(entity_count::<Transfer>(), 2);
}

#[test]
fn test_bigint_arithmetic() {
    // Test that BigInt arithmetic works correctly in native mode
    let a = BigInt::from_u64(1000);
    let b = BigInt::from_u64(337);

    assert_eq!(a.plus(&b).to_string(), "1337");
    assert_eq!(a.minus(&b).to_string(), "663");
    assert_eq!(a.times(&b).to_string(), "337000");
    assert_eq!(a.divided_by(&b).to_string(), "2"); // Integer division
    assert_eq!(a.modulo(&b).to_string(), "326");
    assert_eq!(b.pow(2).to_string(), "113569");
}

#[test]
fn test_bigdecimal_arithmetic() {
    let a = BigDecimal::from_string("100.5");
    let b = BigDecimal::from_string("2.5");

    assert_eq!(a.plus(&b).to_string(), "103.0");
    assert_eq!(a.minus(&b).to_string(), "98.0");
    assert_eq!(a.times(&b).to_string(), "251.25");
}

// ============================================================================
// Call Handler Tests
// ============================================================================

#[test]
fn test_handle_transfer_call_creates_entity() {
    clear_store();

    // Create a test call using CallBuilder
    let call: TransferCall = CallBuilder::new()
        .to(Address::from([0xCCu8; 20]))      // Contract address
        .from(Address::from([0x11u8; 20]))    // Caller address
        .block_number(12345678)
        .block_timestamp(1700000000)
        .transaction_hash([0xABu8; 32])
        .inputs(TransferInputs {
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(500_000_000_000_000_000), // 0.5 ETH
        })
        .outputs(TransferOutputs {
            output0: true,
        })
        .build();

    // Call the handler
    handle_transfer_call(call);

    // Verify that a Transfer entity was created
    // The ID format for call handlers is "call-{tx_hash}"
    let expected_id = "call-0xabababababababababababababababababababababababababababababababab";
    assert_entity_exists::<Transfer>(expected_id);

    // Load and verify the entity
    let transfer = Transfer::load(expected_id).expect("Transfer should exist");
    assert_eq!(transfer.value().to_string(), "500000000000000000");
    assert_eq!(transfer.block_number().to_string(), "12345678");
}

#[test]
fn test_call_builder_defaults() {
    // Test that CallBuilder provides sensible defaults
    let call: TransferCall = CallBuilder::new()
        .inputs(TransferInputs {
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(100),
        })
        .build();

    // Check defaults are reasonable
    assert_eq!(call.to, Address::zero());
    assert_eq!(call.from, Address::zero());
    assert_eq!(call.block.number.to_string(), "1");
    assert_eq!(call.inputs.value.to_string(), "100");
}

// ============================================================================
// Block Builder Tests
// ============================================================================

#[test]
fn test_block_builder() {
    use yogurt_runtime::ethereum::Block;

    // Build a custom block
    let block: Block = BlockBuilder::new()
        .number(15_000_000)
        .timestamp(1660000000)
        .hash([0xAAu8; 32])
        .parent_hash([0xBBu8; 32])
        .author(Address::from([0xCCu8; 20]))
        .gas_used(15_000_000)
        .gas_limit(30_000_000)
        .base_fee_per_gas(20_000_000_000) // 20 gwei
        .build();

    // Verify the block fields
    assert_eq!(block.number.to_string(), "15000000");
    assert_eq!(block.timestamp.to_string(), "1660000000");
    assert_eq!(block.gas_used.to_string(), "15000000");
    assert_eq!(block.gas_limit.to_string(), "30000000");
    assert!(block.base_fee_per_gas.is_some());
    assert_eq!(block.base_fee_per_gas.unwrap().to_string(), "20000000000");
}

#[test]
fn test_block_builder_defaults() {
    use yogurt_runtime::ethereum::Block;

    // Test that BlockBuilder provides sensible defaults
    let block: Block = BlockBuilder::new().build();

    assert_eq!(block.number.to_string(), "1");
    assert_eq!(block.timestamp.to_string(), "1000000000");
    assert_eq!(block.gas_limit.to_string(), "30000000");
    assert!(block.size.is_none());
    assert!(block.base_fee_per_gas.is_none());
}

#[test]
fn test_create_block_helper() {
    use yogurt_runtime::ethereum::Block;

    // Test the convenience function
    let block: Block = create_block(100, 2000000000);

    assert_eq!(block.number.to_string(), "100");
    assert_eq!(block.timestamp.to_string(), "2000000000");
}

#[test]
fn test_create_call_helper() {
    // Test the convenience function
    let call: TransferCall = create_call(
        TransferInputs {
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(1000),
        },
        TransferOutputs { output0: true },
    );

    assert_eq!(call.inputs.value.to_string(), "1000");
    assert!(call.outputs.output0);
}

// ============================================================================
// Crypto Tests
// ============================================================================

#[test]
fn test_keccak256() {
    use yogurt_runtime::crypto;

    // Test with known input
    let hash = crypto::keccak256(b"hello");

    // keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
    assert_eq!(hash.len(), 32);
    assert_eq!(hash.to_hex(), "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8");
}

#[test]
fn test_keccak256_empty() {
    use yogurt_runtime::crypto;

    // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
    let hash = crypto::keccak256(b"");
    assert_eq!(hash.to_hex(), "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
}

#[test]
fn test_keccak256_for_entity_id() {
    use yogurt_runtime::crypto;

    // Common pattern: hashing addresses to create entity IDs
    let from = Address::from([0x11u8; 20]);
    let to = Address::from([0x22u8; 20]);

    let mut data = Vec::new();
    data.extend_from_slice(from.as_bytes());
    data.extend_from_slice(to.as_bytes());

    let hash = crypto::keccak256(&data);
    assert_eq!(hash.len(), 32);

    // Can use the hash as an entity ID
    let id = hash.to_hex();
    assert!(id.starts_with("0x"));
    assert_eq!(id.len(), 66); // "0x" + 64 hex chars
}

// ============================================================================
// Data Source Mocking Tests
// ============================================================================

#[test]
fn test_mock_data_source_address() {
    use yogurt_runtime::data_source;

    clear_data_source_mocks();

    // Default is zero address
    assert_eq!(data_source::address(), Address::zero());

    // Mock the address
    let contract_addr = Address::from([0xABu8; 20]);
    mock_data_source_address(contract_addr.clone());

    assert_eq!(data_source::address(), contract_addr);

    // Clear and verify it resets
    clear_data_source_mocks();
    assert_eq!(data_source::address(), Address::zero());
}

#[test]
fn test_mock_data_source_network() {
    use yogurt_runtime::data_source;

    clear_data_source_mocks();

    // Default is "mainnet"
    assert_eq!(data_source::network(), "mainnet");

    // Mock the network
    mock_data_source_network("goerli");
    assert_eq!(data_source::network(), "goerli");

    mock_data_source_network("arbitrum-one");
    assert_eq!(data_source::network(), "arbitrum-one");

    clear_data_source_mocks();
    assert_eq!(data_source::network(), "mainnet");
}

#[test]
fn test_mock_data_source_context() {
    use yogurt_runtime::data_source;
    use yogurt_runtime::types::{EntityData, Value};

    clear_data_source_mocks();

    // Default is empty
    let ctx = data_source::context();
    assert!(ctx.get("foo").is_none());

    // Mock a context with some data
    let mut context = EntityData::new();
    context.set("poolId", Value::String("pool-123".into()));
    context.set("factory", Value::String("0xfactory".into()));
    mock_data_source_context(context);

    let ctx = data_source::context();
    assert_eq!(ctx.get_string("poolId"), "pool-123");
    assert_eq!(ctx.get_string("factory"), "0xfactory");
}

// ============================================================================
// IPFS Mocking Tests
// ============================================================================

#[test]
fn test_mock_ipfs_cat() {
    use yogurt_runtime::ipfs;

    clear_ipfs_mocks();

    // Without mock, returns None
    assert!(ipfs::cat("QmTest123").is_none());

    // Register mock content
    let json_content = br#"{"name": "Test Token", "symbol": "TEST"}"#;
    mock_ipfs_cat("QmTest123", json_content.to_vec());

    // Now it returns the content
    let content = ipfs::cat("QmTest123").expect("should return mocked content");
    assert_eq!(content.as_slice(), json_content.as_slice());

    // Different CID still returns None
    assert!(ipfs::cat("QmOther456").is_none());

    // Clear and verify
    clear_ipfs_mocks();
    assert!(ipfs::cat("QmTest123").is_none());
}

#[test]
fn test_mock_ipfs_multiple_files() {
    use yogurt_runtime::ipfs;

    clear_ipfs_mocks();

    mock_ipfs_cat("QmFile1", b"content 1");
    mock_ipfs_cat("QmFile2", b"content 2");
    mock_ipfs_cat("QmFile3", b"content 3");

    assert_eq!(ipfs::cat("QmFile1").unwrap().as_slice(), b"content 1");
    assert_eq!(ipfs::cat("QmFile2").unwrap().as_slice(), b"content 2");
    assert_eq!(ipfs::cat("QmFile3").unwrap().as_slice(), b"content 3");
}

// ============================================================================
// JSON Parsing Tests
// ============================================================================

#[test]
fn test_json_parse_object() {
    use yogurt_runtime::json;

    let data = br#"{"name": "Test Token", "symbol": "TEST", "decimals": 18}"#;
    let value = json::from_string(core::str::from_utf8(data).unwrap());

    assert!(value.is_object());
    assert_eq!(value.get("name").unwrap().as_string(), Some("Test Token"));
    assert_eq!(value.get("symbol").unwrap().as_string(), Some("TEST"));
    assert_eq!(value.get("decimals").unwrap().as_i64(), Some(18));
}

#[test]
fn test_json_parse_array() {
    use yogurt_runtime::json;

    let data = r#"[1, 2, 3, "four", true, null]"#;
    let value = json::from_string(data);

    assert!(value.is_array());
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 6);
    assert_eq!(arr[0].as_i64(), Some(1));
    assert_eq!(arr[1].as_i64(), Some(2));
    assert_eq!(arr[2].as_i64(), Some(3));
    assert_eq!(arr[3].as_string(), Some("four"));
    assert_eq!(arr[4].as_bool(), Some(true));
    assert!(arr[5].is_null());
}

#[test]
fn test_json_parse_nested() {
    use yogurt_runtime::json;

    let data = r#"{
        "token": {
            "name": "Wrapped Ether",
            "symbol": "WETH"
        },
        "balances": [100, 200, 300]
    }"#;
    let value = json::from_string(data);

    let token = value.get("token").unwrap();
    assert_eq!(token.get("name").unwrap().as_string(), Some("Wrapped Ether"));
    assert_eq!(token.get("symbol").unwrap().as_string(), Some("WETH"));

    let balances = value.get("balances").unwrap().as_array().unwrap();
    assert_eq!(balances.len(), 3);
    assert_eq!(balances[0].as_i64(), Some(100));
}

#[test]
fn test_json_parse_from_bytes() {
    use yogurt_runtime::json;
    use yogurt_runtime::types::Bytes;

    let data = Bytes::from(br#"{"active": true}"#.to_vec());
    let value = json::from_bytes(&data);

    assert!(value.is_object());
    assert_eq!(value.get("active").unwrap().as_bool(), Some(true));
}

#[test]
fn test_json_parse_invalid() {
    use yogurt_runtime::json;

    let value = json::from_string("not valid json {{{");
    assert!(value.is_null());

    let result = json::try_from_string("not valid json {{{");
    assert!(result.is_err());
}

#[test]
fn test_json_to_bigint() {
    use yogurt_runtime::json;

    // Large numbers that fit in i64
    let value = json::from_string(r#"{"amount": 1000000000000000000}"#);
    let amount = value.get("amount").unwrap().as_big_int().unwrap();
    assert_eq!(amount.to_string(), "1000000000000000000");

    // String representation of big numbers
    let value2 = json::from_string(r#"{"amount": "999999999999999999999999"}"#);
    let amount2 = value2.get("amount").unwrap().as_big_int().unwrap();
    assert_eq!(amount2.to_string(), "999999999999999999999999");
}

// ============================================================================
// DX Features: block_id! Macro Tests
// ============================================================================

#[test]
fn test_block_id_macro() {
    use yogurt_runtime::block_id;
    use yogurt_runtime::ethereum::Block;

    // Build a block
    let block: Block = BlockBuilder::new()
        .number(15_000_000)
        .build();

    // block_id! extracts the block number as a string
    let id = block_id!(block);
    assert_eq!(id, "15000000");
}

// ============================================================================
// DX Features: Entity::exists() Tests
// ============================================================================

#[test]
fn test_entity_exists() {
    clear_store();

    // Initially, entity doesn't exist
    assert!(!Transfer::exists("test-id-123"));

    // Create and save an entity
    let event: TransferEvent = EventBuilder::new()
        .transaction_hash([0x99u8; 32])
        .log_index(0)
        .params(TransferParams {
            from: Address::from([0x11u8; 20]),
            to: Address::from([0x22u8; 20]),
            value: BigInt::from_u64(100),
        })
        .build();

    handle_transfer(event);

    // Now it exists
    let expected_id = "0x9999999999999999999999999999999999999999999999999999999999999999-0";
    assert!(Transfer::exists(expected_id));

    // Other IDs still don't exist
    assert!(!Transfer::exists("other-id"));
}

#[test]
fn test_entity_exists_after_remove() {
    clear_store();

    // Create an entity
    let mut transfer = Transfer::new("remove-test");
    transfer.set_from(Address::from([0x11u8; 20]));
    transfer.set_to(Address::from([0x22u8; 20]));
    transfer.set_value(BigInt::from_u64(500));
    transfer.set_block_number(BigInt::from_u64(1));
    transfer.set_block_timestamp(BigInt::from_u64(1000));
    transfer.set_transaction_hash(Bytes::from([0xAAu8; 32].to_vec()));
    transfer.save();

    assert!(Transfer::exists("remove-test"));

    // Remove it
    Transfer::remove("remove-test");

    // Now it doesn't exist
    assert!(!Transfer::exists("remove-test"));
}

// ============================================================================
// DX Features: Builder Pattern Tests
// ============================================================================

#[test]
fn test_entity_builder_pattern() {
    clear_store();

    // Use builder pattern to create an entity
    let transfer = Transfer::builder("builder-test-1")
        .from(Address::from([0x11u8; 20]))
        .to(Address::from([0x22u8; 20]))
        .value(BigInt::from_u64(1337))
        .block_number(BigInt::from_u64(12345))
        .block_timestamp(BigInt::from_u64(1700000000))
        .transaction_hash(Bytes::from([0xFFu8; 32].to_vec()))
        .build();

    // Verify fields are set correctly
    assert_eq!(transfer.value().to_string(), "1337");
    assert_eq!(transfer.block_number().to_string(), "12345");

    // Save the entity
    transfer.save();
    assert!(Transfer::exists("builder-test-1"));
}

#[test]
fn test_entity_builder_with_save() {
    clear_store();

    // Builder's save() method creates and saves in one go
    Transfer::builder("builder-test-2")
        .from(Address::from([0x33u8; 20]))
        .to(Address::from([0x44u8; 20]))
        .value(BigInt::from_u64(42))
        .block_number(BigInt::from_u64(100))
        .block_timestamp(BigInt::from_u64(2000000000))
        .transaction_hash(Bytes::from([0xABu8; 32].to_vec()))
        .save();

    // Verify entity was saved
    assert!(Transfer::exists("builder-test-2"));
    let loaded = Transfer::load("builder-test-2").unwrap();
    assert_eq!(loaded.value().to_string(), "42");
}

#[test]
fn test_builder_with_automatic_coercion() {
    clear_store();

    // Builder should accept Address directly for Bytes fields
    let from_addr = Address::from([0x55u8; 20]);
    let to_addr = Address::from([0x66u8; 20]);

    let transfer = Transfer::builder("coercion-test")
        .from(from_addr)  // Address → Bytes coercion
        .to(to_addr)      // Address → Bytes coercion
        .value(BigInt::from_u64(999))
        .block_number(BigInt::from_u64(50))
        .block_timestamp(BigInt::from_u64(1500000000))
        .transaction_hash([0xCCu8; 32])  // Array → Bytes coercion
        .build();

    // Verify the addresses were converted correctly
    assert_eq!(transfer.from().as_slice(), &[0x55u8; 20]);
    assert_eq!(transfer.to().as_slice(), &[0x66u8; 20]);
}

// ============================================================================
// DX Features: Address::is_zero() Tests
// ============================================================================

#[test]
fn test_address_is_zero() {
    let zero = Address::zero();
    assert!(zero.is_zero());

    let non_zero = Address::from([0x11u8; 20]);
    assert!(!non_zero.is_zero());

    // Partially zero address is not zero
    let mut partial = [0u8; 20];
    partial[19] = 1;
    let partial_addr = Address::from(partial);
    assert!(!partial_addr.is_zero());
}

// ============================================================================
// DX Features: safe_div Tests
// ============================================================================

#[test]
fn test_bigint_safe_div() {
    let a = BigInt::from_u64(100);
    let b = BigInt::from_u64(3);
    let zero = BigInt::zero();

    // Normal division
    assert_eq!(a.safe_div(&b).to_string(), "33");

    // Division by zero returns zero instead of panic
    assert_eq!(a.safe_div(&zero).to_string(), "0");
    assert_eq!(zero.safe_div(&zero).to_string(), "0");
}

#[test]
fn test_bigdecimal_safe_div() {
    let a = BigDecimal::from_string("100.0");
    let b = BigDecimal::from_string("4.0");
    let zero = BigDecimal::zero();

    // Normal division - may have trailing zeros due to precision
    let result = a.safe_div(&b);
    assert!(result.to_string().starts_with("25"));

    // Division by zero returns zero instead of panic
    assert!(a.safe_div(&zero).is_zero());
}

// ============================================================================
// DX Features: Token Formatting Tests
// ============================================================================

#[test]
fn test_to_decimals() {
    // ETH: 18 decimals
    let wei = BigInt::from_string("1500000000000000000").unwrap();  // 1.5 ETH
    assert_eq!(wei.to_decimals(18), "1.5");

    // Exact whole number
    let one_eth = BigInt::from_string("1000000000000000000").unwrap();
    assert_eq!(one_eth.to_decimals(18), "1");

    // Small amount with leading zeros - trailing zeros are trimmed
    let small = BigInt::from_string("1000000").unwrap();  // 0.000000000001 ETH
    assert_eq!(small.to_decimals(18), "0.000000000001");

    // USDC: 6 decimals
    let usdc = BigInt::from_string("1500000").unwrap();  // 1.5 USDC
    assert_eq!(usdc.to_decimals(6), "1.5");

    // Zero decimals
    let amount = BigInt::from_u64(12345);
    assert_eq!(amount.to_decimals(0), "12345");
}

#[test]
fn test_parse_units() {
    use yogurt_runtime::types::parse_units;

    // ETH: 18 decimals
    let wei = parse_units("1.5", 18);
    assert_eq!(wei.to_string(), "1500000000000000000");

    // Whole number
    let one_eth = parse_units("1", 18);
    assert_eq!(one_eth.to_string(), "1000000000000000000");

    // USDC: 6 decimals
    let usdc = parse_units("1.5", 6);
    assert_eq!(usdc.to_string(), "1500000");

    // High precision truncated
    let precise = parse_units("1.123456789", 6);
    assert_eq!(precise.to_string(), "1123456");  // Truncated to 6 decimals
}

#[test]
fn test_format_units_function() {
    use yogurt_runtime::format_units;

    let wei = BigInt::from_string("2500000000000000000").unwrap();
    assert_eq!(format_units(&wei, 18), "2.5");
}

// ============================================================================
// DX Features: Time-based ID Macros Tests
// ============================================================================

#[test]
fn test_day_id_macro() {
    use yogurt_runtime::day_id;

    // Create an event with a known timestamp
    // 1700000000 seconds = ~19676 days since epoch
    let event: TransferEvent = EventBuilder::new()
        .block_timestamp(1700000000)
        .params(TransferParams {
            from: Address::zero(),
            to: Address::zero(),
            value: BigInt::zero(),
        })
        .build();

    let day = day_id!(event);
    assert_eq!(day, "19675");  // 1700000000 / 86400 = 19675
}

#[test]
fn test_hour_id_macro() {
    use yogurt_runtime::hour_id;

    // Create an event with a known timestamp
    let event: TransferEvent = EventBuilder::new()
        .block_timestamp(1700000000)
        .params(TransferParams {
            from: Address::zero(),
            to: Address::zero(),
            value: BigInt::zero(),
        })
        .build();

    let hour = hour_id!(event);
    assert_eq!(hour, "472222");  // 1700000000 / 3600 = 472222
}
