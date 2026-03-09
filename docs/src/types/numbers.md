# BigInt and BigDecimal

Arbitrary precision numbers for blockchain maths.

## BigInt

`BigInt` represents arbitrary precision integers. Use it for token amounts, block numbers, and any integer that might exceed 64 bits.

### Creating BigInt

```rust
// From primitives
let a = BigInt::from_i32(42);
let b = BigInt::from_u64(1_000_000_000_000_000_000);

// From string (for very large numbers)
let c = BigInt::from_string("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap();

// From bytes (big-endian)
let d = BigInt::from_bytes(&[0x01, 0x00]);  // 256

// Zero
let zero = BigInt::zero();
```

### Arithmetic Operations

```rust
let a = BigInt::from_i32(100);
let b = BigInt::from_i32(30);

// Basic operations
let sum = a.plus(&b);           // 130
let diff = a.minus(&b);         // 70
let product = a.times(&b);      // 3000
let quotient = a.divided_by(&b); // 3

// Rust operators also work
let sum = &a + &b;
let diff = &a - &b;
let product = &a * &b;
let quotient = &a / &b;

// Modulo
let remainder = a.mod_op(&b);   // 10

// Power
let squared = a.pow(2);         // 10000

// Absolute value
let neg = BigInt::from_i32(-42);
let abs = neg.abs();            // 42

// Square root (integer)
let sqrt = BigInt::from_i32(100).sqrt();  // 10
```

### Safe Division

Division by zero normally panics. Use `safe_div` to return zero instead:

```rust
let a = BigInt::from_i32(100);
let b = BigInt::zero();

let result = a.safe_div(&b);  // Returns BigInt::zero(), no panic
```

### Comparison

```rust
let a = BigInt::from_i32(100);
let b = BigInt::from_i32(50);

// Comparison operators
if a > b { /* ... */ }
if a >= b { /* ... */ }
if a < b { /* ... */ }
if a <= b { /* ... */ }
if a == b { /* ... */ }

// Method style
if a.gt(&b) { /* ... */ }
if a.ge(&b) { /* ... */ }
if a.lt(&b) { /* ... */ }
if a.le(&b) { /* ... */ }

// Check zero/negative
if a.is_zero() { /* ... */ }
```

### Conversion

```rust
let big = BigInt::from_i32(42);

// To string
let s: String = big.to_string();  // "42"

// To primitives (may overflow!)
let n: i32 = big.to_i32();
let n: u64 = big.to_u64();

// To bytes
let bytes: Vec<u8> = big.to_bytes();

// To signed bytes (two's complement)
let signed: Vec<u8> = big.to_signed_bytes();
```

## BigDecimal

`BigDecimal` represents arbitrary precision decimals. Use it for prices, percentages, and any value requiring decimal precision.

### Creating BigDecimal

```rust
// From BigInt
let amount = BigInt::from_u64(1_500_000_000_000_000_000);
let eth = BigDecimal::from_big_int(&amount);  // 1500000000000000000

// From string
let price = BigDecimal::from_string("1234.5678").unwrap();

// Zero
let zero = BigDecimal::zero();
```

### Arithmetic Operations

```rust
let a = BigDecimal::from_string("100.5").unwrap();
let b = BigDecimal::from_string("50.25").unwrap();

// Basic operations
let sum = a.plus(&b);           // 150.75
let diff = a.minus(&b);         // 50.25
let product = a.times(&b);      // 5050.125
let quotient = a.divided_by(&b); // ~2.0

// Rust operators
let sum = &a + &b;
let diff = &a - &b;
let product = &a * &b;
let quotient = &a / &b;
```

### Safe Division

```rust
let a = BigDecimal::from_string("100").unwrap();
let b = BigDecimal::zero();

let result = a.safe_div(&b);  // Returns BigDecimal::zero()
```

### Truncation

Limit decimal places:

```rust
let value = BigDecimal::from_string("3.141592653589793").unwrap();
let truncated = value.truncate(4);  // 3.1415
```

### Comparison

```rust
let a = BigDecimal::from_string("100.5").unwrap();
let b = BigDecimal::from_string("50.25").unwrap();

if a > b { /* ... */ }
if a.gt(&b) { /* ... */ }
if a.is_zero() { /* ... */ }
```

### Conversion

```rust
let decimal = BigDecimal::from_string("123.456").unwrap();

// To string
let s: String = decimal.to_string();  // "123.456"
```

## Common Patterns

### Token Amount Calculations

```rust
// Calculate percentage
fn calculate_fee(amount: &BigInt, fee_percent: &BigDecimal) -> BigInt {
    let amount_dec = BigDecimal::from_big_int(amount);
    let fee = amount_dec.times(fee_percent).divided_by(&BigDecimal::from_string("100").unwrap());
    // Convert back to BigInt (truncates decimals)
    BigInt::from_string(&fee.truncate(0).to_string()).unwrap()
}

// Safe ratio calculation
fn calculate_price(reserve0: &BigInt, reserve1: &BigInt) -> BigDecimal {
    if reserve0.is_zero() {
        return BigDecimal::zero();
    }
    let r0 = BigDecimal::from_big_int(reserve0);
    let r1 = BigDecimal::from_big_int(reserve1);
    r1.divided_by(&r0)
}
```

### Accumulating Values

```rust
// Running total
let mut total = BigInt::zero();
for amount in amounts {
    total = total.plus(&amount);
}

// Or with operators
let mut total = BigInt::zero();
for amount in amounts {
    total = &total + &amount;
}
```
