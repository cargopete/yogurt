# Token Formatting

Utilities for converting between raw token amounts (wei) and human-readable decimals (ETH).

## format_units

Converts a raw integer amount to a decimal string with the specified number of decimal places.

```rust
use yogurt_runtime::format_units;

let wei = BigInt::from_string("1500000000000000000").unwrap();
let eth = format_units(&wei, 18);  // "1.5"
```

### Examples

```rust
// ETH (18 decimals)
let wei = BigInt::from_string("1000000000000000000").unwrap();
format_units(&wei, 18);  // "1"

let wei = BigInt::from_string("1500000000000000000").unwrap();
format_units(&wei, 18);  // "1.5"

let wei = BigInt::from_string("123456789000000000").unwrap();
format_units(&wei, 18);  // "0.123456789"

// USDC (6 decimals)
let raw = BigInt::from_string("1500000").unwrap();
format_units(&raw, 6);  // "1.5"

// Small amounts
let wei = BigInt::from_string("1000000").unwrap();
format_units(&wei, 18);  // "0.000000000001"

// Zero decimals
let amount = BigInt::from_i32(42);
format_units(&amount, 0);  // "42"
```

### Trailing Zeros

Trailing zeros are automatically trimmed:

```rust
let wei = BigInt::from_string("1000000000000000000").unwrap();
format_units(&wei, 18);  // "1" (not "1.000000000000000000")
```

## parse_units

Converts a human-readable decimal string to a raw integer amount.

```rust
use yogurt_runtime::prelude::parse_units;

let wei = parse_units("1.5", 18);
// wei = 1500000000000000000
```

### Examples

```rust
// ETH (18 decimals)
parse_units("1", 18);      // 1000000000000000000
parse_units("1.5", 18);    // 1500000000000000000
parse_units("0.001", 18);  // 1000000000000000

// USDC (6 decimals)
parse_units("1", 6);       // 1000000
parse_units("1.5", 6);     // 1500000
parse_units("100.50", 6);  // 100500000

// Integer input
parse_units("42", 18);     // 42000000000000000000

// Zero
parse_units("0", 18);      // 0
```

### Negative Values

Both functions handle negative values:

```rust
let neg = BigInt::from_string("-1500000000000000000").unwrap();
format_units(&neg, 18);  // "-1.5"

parse_units("-1.5", 18);  // -1500000000000000000
```

## BigInt::to_decimals

The `format_units` function is a convenience wrapper around `BigInt::to_decimals`:

```rust
let wei = BigInt::from_string("1500000000000000000").unwrap();

// These are equivalent:
let eth = format_units(&wei, 18);
let eth = wei.to_decimals(18);
```

## Common Patterns

### Display Token Balance

```rust
fn format_token_balance(balance: &BigInt, decimals: u8, symbol: &str) -> String {
    format!("{} {}", format_units(balance, decimals), symbol)
}

// Usage
let balance = BigInt::from_string("1500000000000000000").unwrap();
let display = format_token_balance(&balance, 18, "ETH");  // "1.5 ETH"
```

### Parse User Input

```rust
fn parse_token_amount(input: &str, decimals: u8) -> Result<BigInt, &'static str> {
    // Validate input
    if input.is_empty() {
        return Err("Amount cannot be empty");
    }

    Ok(parse_units(input, decimals))
}
```

### Convert Between Token Decimals

```rust
fn convert_decimals(amount: &BigInt, from_decimals: u8, to_decimals: u8) -> BigInt {
    if from_decimals == to_decimals {
        return amount.clone();
    }

    if from_decimals > to_decimals {
        // Reduce precision
        let factor = BigInt::from_i32(10).pow((from_decimals - to_decimals) as u8);
        amount.divided_by(&factor)
    } else {
        // Increase precision
        let factor = BigInt::from_i32(10).pow((to_decimals - from_decimals) as u8);
        amount.times(&factor)
    }
}

// Example: USDC (6) to 18 decimals for consistent storage
let usdc_amount = BigInt::from_i32(1_500_000);  // 1.5 USDC
let normalized = convert_decimals(&usdc_amount, 6, 18);
// normalized = 1500000000000000000
```

### Price Calculations

```rust
fn calculate_token_value_usd(
    amount: &BigInt,
    token_decimals: u8,
    price_usd: &BigDecimal,
) -> BigDecimal {
    let amount_decimal = BigDecimal::from_big_int(amount);
    let divisor = BigDecimal::from_big_int(
        &BigInt::from_i32(10).pow(token_decimals)
    );

    amount_decimal
        .divided_by(&divisor)
        .times(price_usd)
}
```

## Comparison with ethers.js

| ethers.js | yogurt |
|-----------|--------|
| `ethers.formatUnits(wei, 18)` | `format_units(&wei, 18)` |
| `ethers.parseUnits("1.5", 18)` | `parse_units("1.5", 18)` |
| `ethers.formatEther(wei)` | `format_units(&wei, 18)` |
| `ethers.parseEther("1.5")` | `parse_units("1.5", 18)` |
