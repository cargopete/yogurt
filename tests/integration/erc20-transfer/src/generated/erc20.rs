//! Auto-generated ERC-20 event types from ABI

use yogurt_runtime::prelude::*;
use yogurt_runtime::asc::{read_u32_at, AscPtr};

/// Transfer(address indexed from, address indexed to, uint256 value)
pub struct TransferParams {
    pub from: Address,
    pub to: Address,
    pub value: BigInt,
}

/// Transfer event with full context
pub type TransferEvent = Event<TransferParams>;

// Field offsets for TransferParams in AS memory
// The params are stored as an Array of EventParam, which we need to decode
mod param_offsets {
    pub const FROM: usize = 0;  // First param pointer
    pub const TO: usize = 4;    // Second param pointer
    pub const VALUE: usize = 8; // Third param pointer
}

// EventParam layout:
// - name: AscPtr<String> (4 bytes)
// - value: AscPtr<EthereumValue> (4 bytes)
mod event_param {
    pub const NAME: usize = 0;
    pub const VALUE: usize = 4;
}

// EthereumValue (wrapped value) layout:
// - kind: i32 (4 bytes)
// - data: varies based on kind

#[cfg(target_arch = "wasm32")]
impl FromAscPtr for TransferParams {
    fn from_asc_ptr(ptr: u32) -> Self {
        use yogurt_runtime::asc::{asc_to_bytes, AscArrayHeader};

        if ptr == 0 {
            return TransferParams {
                from: Address::zero(),
                to: Address::zero(),
                value: BigInt::zero(),
            };
        }

        unsafe {
            // ptr points to an Array of EventParam
            let array_header = ptr as *const AscArrayHeader;
            let buffer_ptr = (*array_header).buffer;
            let _length = (*array_header).length;

            // Read each EventParam pointer from the buffer
            // EventParam[0] = from (address)
            let param0_ptr = read_u32_at(buffer_ptr, 0);
            let from_value_ptr = read_u32_at(param0_ptr, event_param::VALUE);
            // The value for address is stored directly as bytes
            let from_bytes = asc_to_bytes(AscPtr::new(from_value_ptr));
            let from = Address::from(from_bytes.as_slice());

            // EventParam[1] = to (address)
            let param1_ptr = read_u32_at(buffer_ptr, 4);
            let to_value_ptr = read_u32_at(param1_ptr, event_param::VALUE);
            let to_bytes = asc_to_bytes(AscPtr::new(to_value_ptr));
            let to = Address::from(to_bytes.as_slice());

            // EventParam[2] = value (uint256)
            let param2_ptr = read_u32_at(buffer_ptr, 8);
            let value_ptr = read_u32_at(param2_ptr, event_param::VALUE);
            let value = BigInt::from_ptr(AscPtr::new(value_ptr));

            TransferParams { from, to, value }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl FromAscPtr for TransferParams {
    fn from_asc_ptr(_ptr: u32) -> Self {
        TransferParams {
            from: Address::zero(),
            to: Address::zero(),
            value: BigInt::zero(),
        }
    }
}
