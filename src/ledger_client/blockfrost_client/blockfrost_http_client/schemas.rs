#![allow(unused)]
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct Genesis {
    active_slots_coefficient: f32,
    update_quorum: u32,
    max_lovelace_supply: String,
    network_magic: u32,
    epoch_length: u32,
    system_start: u32,
    slots_per_kes_period: u32,
    slot_length: u32,
    max_kes_evolutions: u32,
    security_param: u32,
}

#[derive(Deserialize, Debug)]
pub struct UTxO {
    tx_hash: String,
    output_index: u32,
    amount: Vec<Value>,
    block: String,
    data_hash: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Value {
    unit: String,
    quantity: String,
}

#[derive(Deserialize, Debug)]
pub struct AddressInfo {
    address: String,
    amount: Vec<Value>,
    stake_address: Option<String>,
    r#type: String,
    script: bool,
}

#[derive(Deserialize, Debug)]
pub struct Address {
    address: String,
}

impl Address {
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[derive(Deserialize, Debug)]
pub struct AccountAssocAddrTotal {
    stake_addr: String,
    received_sum: Vec<Value>,
    sent_sum: Vec<Value>,
    tx_count: u32,
}
