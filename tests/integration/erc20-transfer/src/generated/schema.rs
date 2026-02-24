//! Auto-generated entity types from schema.graphql

use yogurt_runtime::prelude::*;
use yogurt_runtime::store;
use yogurt_runtime::types::{EntityData, Value};

/// Transfer entity
pub struct Transfer {
    data: EntityData,
}

impl Transfer {
    pub fn new(id: impl Into<alloc::string::String>) -> Self {
        let mut data = EntityData::new();
        data.set("id", Value::String(id.into()));
        Self { data }
    }

    // Getters
    pub fn id(&self) -> &str {
        self.data.get_string("id")
    }

    pub fn from(&self) -> Bytes {
        self.data.get_bytes("from")
    }

    pub fn to(&self) -> Bytes {
        self.data.get_bytes("to")
    }

    pub fn value(&self) -> BigInt {
        self.data.get_bigint("value")
    }

    pub fn block_number(&self) -> BigInt {
        self.data.get_bigint("blockNumber")
    }

    pub fn block_timestamp(&self) -> BigInt {
        self.data.get_bigint("blockTimestamp")
    }

    pub fn transaction_hash(&self) -> Bytes {
        self.data.get_bytes("transactionHash")
    }

    // Setters
    pub fn set_from(&mut self, val: impl Into<Bytes>) {
        self.data.set("from", Value::Bytes(val.into()));
    }

    pub fn set_to(&mut self, val: impl Into<Bytes>) {
        self.data.set("to", Value::Bytes(val.into()));
    }

    pub fn set_value(&mut self, val: impl Into<BigInt>) {
        self.data.set("value", Value::BigInt(val.into()));
    }

    pub fn set_block_number(&mut self, val: impl Into<BigInt>) {
        self.data.set("blockNumber", Value::BigInt(val.into()));
    }

    pub fn set_block_timestamp(&mut self, val: impl Into<BigInt>) {
        self.data.set("blockTimestamp", Value::BigInt(val.into()));
    }

    pub fn set_transaction_hash(&mut self, val: impl Into<Bytes>) {
        self.data.set("transactionHash", Value::Bytes(val.into()));
    }
}

impl Entity for Transfer {
    const ENTITY_TYPE: &'static str = "Transfer";

    fn id(&self) -> &str {
        self.data.get_string("id")
    }

    fn save(&self) {
        store::set(Self::ENTITY_TYPE, self.id(), &self.data);
    }

    fn load(id: &str) -> Option<Self> {
        store::get(Self::ENTITY_TYPE, id).map(|data| Self { data })
    }

    fn remove(id: &str) {
        store::remove(Self::ENTITY_TYPE, id);
    }
}
