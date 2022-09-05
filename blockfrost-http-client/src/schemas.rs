#![allow(unused)]

use serde::Deserialize;
use std::collections::HashMap;

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

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct CostModels {
    PlutusV1: HashMap<String, u64>,
    PlutusV2: HashMap<String, u64>,
}

// #[derive(Deserialize, Debug)]
// pub struct CostModel {
//     addInteger_cpu_arguments_intercept: u32,
//     addInteger_cpu_arguments_slope: u32,
// }

#[derive(Deserialize, Debug)]
pub struct ProtocolParams {
    epoch: u32,
    min_fee_a: u32,
    min_fee_b: u32,
    max_block_size: u32,
    max_tx_size: u32,
    max_block_header_size: u32,
    key_deposit: String,
    pool_deposit: String,
    e_max: u32,
    n_opt: u32,
    a0: f32,
    rho: f32,
    tau: f32,
    decentralisation_param: f32,
    extra_entropy: Option<serde_json::Value>,
    protocol_major_ver: u32,
    protocol_minor_ver: u32,
    min_utxo: String,
    min_pool_cost: String,
    nonce: String,
    price_mem: f32,
    price_step: f32,
    max_tx_ex_mem: String,
    max_tx_ex_steps: String,
    max_block_ex_mem: String,
    max_block_ex_steps: String,
    max_val_size: String,
    collateral_percent: u32,
    max_collateral_inputs: u32,
    coins_per_utxo_size: String,
    coins_per_utxo_word: String,
    cost_models: CostModels,
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
    pub fn tx_hash(&self) -> &str {
        &self.tx_hash
    }

    pub fn output_index(&self) -> u32 {
        self.output_index
    }

    pub fn amount(&self) -> &Vec<Value> {
        &self.amount
    }
}

#[derive(Deserialize, Debug)]
pub struct Value {
    unit: String,
    quantity: String,
}

impl Value {
    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn quantity(&self) -> &str {
        &self.quantity
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

#[derive(Deserialize, Debug)]
pub struct Fault {
    code: String,
    string: String,
}

#[derive(Deserialize, Debug)]
pub struct EvaluateTxResult {
    methodname: Option<String>,
    reflection: HashMap<String, String>,
    result: serde_json::Value,
    fault: Option<HashMap<String, String>>,
    servicename: String,
    r#type: String,
    version: String,
}

//"result": Object({
//         "EvaluationFailure": Object({
//             "ScriptFailures": Object({
//                 "spend:0": Array([
//                     Object({
//                         "extraRedeemers": Array([
//                             String(
//                                 "spend:0",
//                             ),
//                         ]),
//                     }),
//                 ]),
//             }),
//         }),
//     }),
#[derive(Deserialize, Debug)]
pub struct EvaluationFailure {}
