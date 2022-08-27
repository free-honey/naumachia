#![allow(unused)]
use crate::output::{Output, OutputId};
use crate::values::Values;
use crate::{address, PolicyId};
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

impl UTxO {
    pub fn into_nau_output<Datum>(&self, owner: &address::Address) -> Output<Datum> {
        let tx_hash = self.tx_hash.to_owned();
        let index = self.output_index.to_owned();
        let mut values = Values::default();
        self.amount
            .iter()
            .map(|value| value.as_nau_value())
            .for_each(|(policy_id, amount)| values.add_one_value(&policy_id, amount));
        Output::new_wallet(tx_hash, index, owner.to_owned(), values)
    }
}

#[derive(Deserialize, Debug)]
pub struct Value {
    unit: String,
    quantity: String,
}

impl Value {
    pub fn as_nau_value(&self) -> (PolicyId, u64) {
        let policy_id = match self.unit.as_str() {
            "lovelace" => PolicyId::ADA,
            native_token => {
                let policy = &native_token[..56]; // TODO: Use the rest as asset info
                PolicyId::native_token(policy)
            }
        };
        let amount = self.quantity.parse().unwrap(); // TODO: unwrap
        (policy_id, amount)
    }
}

impl From<Value> for (PolicyId, u64) {
    fn from(value: Value) -> Self {
        todo!()
    }
}
#[derive(Deserialize, Debug)]
pub struct AddressInfo {
    address: String,
    amount: Vec<Value>,
    stake_address: Option<String>,
    r#type: String,
    script: bool,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Address {
    address: String,
}

impl From<Address> for address::Address {
    fn from(addr: Address) -> Self {
        address::Address::new(addr.as_string())
    }
}

impl Address {
    pub fn as_string(&self) -> &str {
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
