//! Uniswap V2 event handlers

use alloc::string::ToString;
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

    // Load or create Factory entity
    let factory_id = FACTORY_ADDRESS.to_string();
    let mut factory = Factory::load(&factory_id).unwrap_or_else(|| {
        let mut f = Factory::new(factory_id);
        f.set_pair_count(BigInt::from(0));
        f.set_total_volume_eth(BigDecimal::zero());
        f.set_total_liquidity_eth(BigDecimal::zero());
        f
    });

    // Increment pair count
    let new_count = factory.pair_count() + BigInt::from(1);
    factory.set_pair_count(new_count);
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
fn fetch_or_create_token(address: &Address) -> Token {
    let token_id = address.to_hex();

    // Try to load existing token
    if let Some(token) = Token::load(&token_id) {
        return token;
    }

    // Fetch metadata from the token contract
    let contract = ERC20::bind(address.clone());

    let symbol = contract.try_symbol().unwrap_or_else(|| "???".to_string());
    let name = contract.try_name().unwrap_or_else(|| "Unknown".to_string());
    let decimals = BigInt::from(contract.try_decimals().unwrap_or(18));
    let total_supply = contract.try_total_supply().unwrap_or_else(BigInt::zero);

    let mut token = Token::new(token_id);
    token.set_symbol(symbol);
    token.set_name(name);
    token.set_decimals(decimals);
    token.set_total_supply(total_supply);
    token.set_trade_volume(BigDecimal::zero());
    token.set_tx_count(BigInt::from(0));
    token.save();

    token
}

/// Handle Swap event from a Pair contract.
#[handler]
pub fn handle_swap(event: SwapEvent) {
    let pair_address = event.address;
    let pair_id = pair_address.to_hex();

    // Create immutable Swap entity
    let swap_id = alloc::format!(
        "{}-{}",
        event.transaction.hash.to_hex(),
        event.log_index.to_string()
    );

    let mut swap = SwapEntity::new(swap_id);
    swap.set_pair(pair_id.clone());
    swap.set_timestamp(event.block.timestamp);
    swap.set_sender(Bytes::from(event.params.sender.as_bytes()));
    swap.set_amount0_in(BigDecimal::from_big_int(&event.params.amount0_in));
    swap.set_amount1_in(BigDecimal::from_big_int(&event.params.amount1_in));
    swap.set_amount0_out(BigDecimal::from_big_int(&event.params.amount0_out));
    swap.set_amount1_out(BigDecimal::from_big_int(&event.params.amount1_out));
    swap.set_to(Bytes::from(event.params.to.as_bytes()));
    swap.set_log_index(event.log_index);
    swap.set_transaction(event.transaction.hash);
    swap.save();

    // Update pair tx count
    if let Some(mut pair) = Pair::load(&pair_id) {
        pair.set_tx_count(pair.tx_count() + BigInt::from(1));
        pair.save();
    }
}

/// Handle Mint (add liquidity) event from a Pair contract.
#[handler]
pub fn handle_mint(event: MintEvent) {
    let pair_address = event.address;
    let pair_id = pair_address.to_hex();

    let mint_id = alloc::format!(
        "{}-{}",
        event.transaction.hash.to_hex(),
        event.log_index.to_string()
    );

    // For Mint, we need to get the 'to' address from the transaction
    // In a real subgraph, you'd extract this from transaction input or receipt
    let to = event.transaction.from.clone(); // Simplification: use tx sender

    let mut mint = MintEntity::new(mint_id);
    mint.set_pair(pair_id.clone());
    mint.set_timestamp(event.block.timestamp);
    mint.set_sender(Bytes::from(event.params.sender.as_bytes()));
    mint.set_amount0(BigDecimal::from_big_int(&event.params.amount0));
    mint.set_amount1(BigDecimal::from_big_int(&event.params.amount1));
    mint.set_to(Bytes::from(to.as_bytes()));
    mint.set_log_index(event.log_index);
    mint.set_transaction(event.transaction.hash);
    mint.save();

    // Update pair tx count
    if let Some(mut pair) = Pair::load(&pair_id) {
        pair.set_tx_count(pair.tx_count() + BigInt::from(1));
        pair.save();
    }
}

/// Handle Burn (remove liquidity) event from a Pair contract.
#[handler]
pub fn handle_burn(event: BurnEvent) {
    let pair_address = event.address;
    let pair_id = pair_address.to_hex();

    let burn_id = alloc::format!(
        "{}-{}",
        event.transaction.hash.to_hex(),
        event.log_index.to_string()
    );

    let mut burn = BurnEntity::new(burn_id);
    burn.set_pair(pair_id.clone());
    burn.set_timestamp(event.block.timestamp);
    burn.set_sender(Bytes::from(event.params.sender.as_bytes()));
    burn.set_amount0(BigDecimal::from_big_int(&event.params.amount0));
    burn.set_amount1(BigDecimal::from_big_int(&event.params.amount1));
    burn.set_to(Bytes::from(event.params.to.as_bytes()));
    burn.set_log_index(event.log_index);
    burn.set_transaction(event.transaction.hash);
    burn.save();

    // Update pair tx count
    if let Some(mut pair) = Pair::load(&pair_id) {
        pair.set_tx_count(pair.tx_count() + BigInt::from(1));
        pair.save();
    }
}

/// Handle Sync event from a Pair contract.
///
/// Updates the pair's reserves with the latest values.
#[handler]
pub fn handle_sync(event: SyncEvent) {
    let pair_address = event.address;
    let pair_id = pair_address.to_hex();

    if let Some(mut pair) = Pair::load(&pair_id) {
        pair.set_reserve0(BigDecimal::from_big_int(&event.params.reserve0));
        pair.set_reserve1(BigDecimal::from_big_int(&event.params.reserve1));
        pair.save();
    }
}
