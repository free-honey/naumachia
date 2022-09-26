#![allow(unused)]

use crate::error::*;
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
    output_index: u64,
    amount: Vec<Value>,
    block: String,
    data_hash: Option<String>,
    inline_datum: Option<serde_json::Value>,
    reference_script_hash: Option<String>,
}

impl UTxO {
    pub fn tx_hash(&self) -> &str {
        &self.tx_hash
    }

    pub fn data_hash(&self) -> &Option<String> {
        &self.data_hash
    }

    // I don't think this is right
    pub fn with_data(&self, data: Option<serde_json::Value>) -> UTxOWithData {
        UTxOWithData {
            tx_hash: self.tx_hash.clone(),
            output_index: self.output_index,
            amount: self.amount.clone(),
            block: self.block.clone(),
            data,
        }
    }

    pub fn block(&self) -> &str {
        &self.block
    }

    pub fn output_index(&self) -> u64 {
        self.output_index
    }

    pub fn amount(&self) -> &Vec<Value> {
        &self.amount
    }
}

#[derive(Deserialize, Debug)]
pub struct UTxOWithData {
    tx_hash: String,
    output_index: u64,
    amount: Vec<Value>,
    block: String,
    data: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Value {
    pub(crate) unit: String,
    pub(crate) quantity: String,
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
    result: Option<Success>,
    fault: Option<HashMap<String, String>>,
    servicename: String,
    r#type: String,
    version: String,
}

impl EvaluateTxResult {
    pub fn get_spends(&self) -> Result<HashMap<u64, Spend>> {
        let mut spends = HashMap::new();
        if let Some(result) = &self.result {
            if let Some(inner) = result.evalutation_result.as_object() {
                let keys = inner.keys();
                for key in keys {
                    if key.contains("spend:") {
                        if let Some(val) = inner.get(key) {
                            let index_str = key
                                .split(':')
                                .collect::<Vec<_>>()
                                .get(1)
                                .unwrap()
                                .to_owned();
                            let index = index_str
                                .parse::<u64>()
                                .map_err(|e| Error::EvaluateTxResult(Box::new(e)))?;
                            let serialized = serde_json::to_string(val)
                                .map_err(|e| Error::EvaluateTxResult(Box::new(e)))?;
                            let deserialized = serde_json::from_str(&serialized).unwrap();
                            spends.insert(index, deserialized);
                        }
                    }
                }
            }
        }
        Ok(spends)
    }
}

#[derive(Deserialize, Debug)]
pub struct Success {
    #[serde(rename = "EvaluationResult")]
    evalutation_result: serde_json::Value,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Spend {
    memory: u64,
    steps: u64,
}

impl Spend {
    pub fn memory(&self) -> u64 {
        self.memory
    }
    pub fn steps(&self) -> u64 {
        self.steps
    }
}

#[derive(Deserialize, Debug)]
pub struct TxSubmitResult(String);

impl TxSubmitResult {
    pub fn tx_id(&self) -> &str {
        &self.0
    }
}
