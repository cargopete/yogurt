//! Uniswap V2 event handlers
//!
//! This example showcases yogurt's developer experience improvements:
//! - `log_id!` macro for entity IDs
//! - `load_or_create`, `update`, `upsert` entity helpers
//! - Automatic `Address → Bytes` coercion
//! - `Entity::exists()` for existence checks

use yogurt_runtime::prelude::*;
use yogurt_macros::handler;

use crate::generated::{
    // Entities
    Factory, Pair, Token, Swap as SwapEntity, Mint as MintEntity, Burn as BurnEntity,
    // Events
    PairCreatedEvent, SwapEvent, MintEvent, BurnEvent, SyncEvent,
    // Contract bindings
    ERC20,
    // Templates
    templates::Pair as PairTemplate,
};

/// Factory address constant
const FACTORY_ADDRESS: &str = "0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f";

/// Handle PairCreated event from the Factory contract.
///
/// Creates Token entities if they don't exist, creates a Pair entity,
/// and spawns a data source template to watch the new pair contract.
#[handler]
pub fn handle_pair_created(event: PairCreatedEvent) {
    let token0_address = event.params.token0;
    let token1_address = event.params.token1;
    let pair_address = event.params.pair;

    // Load or create Factory entity using the helper method
    let mut factory = Factory::load_or_create(FACTORY_ADDRESS, |f| {
        f.set_pair_count(BigInt::zero());
        f.set_total_volume_eth(BigDecimal::zero());
        f.set_total_liquidity_eth(BigDecimal::zero());
    });

    // Increment pair count
    factory.set_pair_count(factory.pair_count() + BigInt::from(1));
    factory.save();

    // Create or load Token entities
    let token0 = fetch_or_create_token(&token0_address);
    let token1 = fetch_or_create_token(&token1_address);

    // Create Pair entity
    let pair_id = pair_address.to_hex();
    let mut pair = Pair::new(pair_id);
    pair.set_token0(token0.id());
    pair.set_token1(token1.id());
    pair.set_reserve0(BigDecimal::zero());
    pair.set_reserve1(BigDecimal::zero());
    pair.set_total_supply(BigDecimal::zero());
    pair.set_tx_count(BigInt::from(0));
    pair.set_created_at_timestamp(event.block.timestamp);
    pair.set_created_at_block_number(event.block.number);
    pair.save();

    // Spawn a data source template to watch the new pair
    PairTemplate::create(&pair_address);

    yogurt_runtime::log_info!(
        "New pair created: {} (tokens: {}, {})",
        pair_address.to_hex(),
        token0_address.to_hex(),
        token1_address.to_hex()
    );
}

/// Fetch token metadata from chain or create a new Token entity.
///
/// Uses `exists()` check and `load_or_create` to simplify the pattern.
fn fetch_or_create_token(address: &Address) -> Token {
    let token_id = address.to_hex();

    // Check if token already exists - if so, just load and return
    if Token::exists(&token_id) {
        return Token::load(&token_id).unwrap();
    }

    // Fetch metadata from the token contract
    let contract = ERC20::bind(address.clone());

    // Create new token with fetched metadata
    let mut token = Token::new(&token_id);
    token.set_symbol(contract.try_symbol().unwrap_or_else(|| "???".into()));
    token.set_name(contract.try_name().unwrap_or_else(|| "Unknown".into()));
    token.set_decimals(BigInt::from(contract.try_decimals().unwrap_or(18)));
    token.set_total_supply(contract.try_total_supply().unwrap_or_else(BigInt::zero));
    token.set_trade_volume(BigDecimal::zero());
    token.set_tx_count(BigInt::zero());
    token.save();

    token
}

/// Handle Swap event from a Pair contract.
#[handler]
pub fn handle_swap(event: SwapEvent) {
    let pair_id = event.address.to_hex();

    // Create immutable Swap entity using builder pattern
    SwapEntity::builder(log_id!(event))
        .pair(&pair_id)
        .timestamp(event.block.timestamp)
        .sender(event.params.sender)  // Address → Bytes coercion is automatic
        .amount0_in(BigDecimal::from_big_int(&event.params.amount0_in))
        .amount1_in(BigDecimal::from_big_int(&event.params.amount1_in))
        .amount0_out(BigDecimal::from_big_int(&event.params.amount0_out))
        .amount1_out(BigDecimal::from_big_int(&event.params.amount1_out))
        .to(event.params.to)
        .log_index(event.log_index)
        .transaction(event.transaction.hash)
        .save();

    // Update pair tx count using Entity::update()
    Pair::update(&pair_id, |p| {
        p.set_tx_count(p.tx_count() + BigInt::from(1));
    });
}

/// Handle Mint (add liquidity) event from a Pair contract.
#[handler]
pub fn handle_mint(event: MintEvent) {
    let pair_id = event.address.to_hex();

    // For Mint, we need to get the 'to' address from the transaction
    // In a real subgraph, you'd extract this from transaction input or receipt
    let to = event.transaction.from.clone(); // Simplification: use tx sender

    // Create Mint entity using builder pattern
    MintEntity::builder(log_id!(event))
        .pair(&pair_id)
        .timestamp(event.block.timestamp)
        .sender(event.params.sender)  // Address → Bytes coercion is automatic
        .amount0(BigDecimal::from_big_int(&event.params.amount0))
        .amount1(BigDecimal::from_big_int(&event.params.amount1))
        .to(to)
        .log_index(event.log_index)
        .transaction(event.transaction.hash)
        .save();

    // Update pair tx count using Entity::update()
    Pair::update(&pair_id, |p| {
        p.set_tx_count(p.tx_count() + BigInt::from(1));
    });
}

/// Handle Burn (remove liquidity) event from a Pair contract.
#[handler]
pub fn handle_burn(event: BurnEvent) {
    let pair_id = event.address.to_hex();

    // Create Burn entity using builder pattern
    BurnEntity::builder(log_id!(event))
        .pair(&pair_id)
        .timestamp(event.block.timestamp)
        .sender(event.params.sender)  // Address → Bytes coercion is automatic
        .amount0(BigDecimal::from_big_int(&event.params.amount0))
        .amount1(BigDecimal::from_big_int(&event.params.amount1))
        .to(event.params.to)
        .log_index(event.log_index)
        .transaction(event.transaction.hash)
        .save();

    // Update pair tx count using Entity::update()
    Pair::update(&pair_id, |p| {
        p.set_tx_count(p.tx_count() + BigInt::from(1));
    });
}

/// Handle Sync event from a Pair contract.
///
/// Updates the pair's reserves with the latest values.
#[handler]
pub fn handle_sync(event: SyncEvent) {
    let pair_id = event.address.to_hex();

    // Update reserves using Entity::update() - concise one-liner pattern
    Pair::update(&pair_id, |p| {
        p.set_reserve0(BigDecimal::from_big_int(&event.params.reserve0));
        p.set_reserve1(BigDecimal::from_big_int(&event.params.reserve1));
    });
}
