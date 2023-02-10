use crate::scripts::ExecutionCost;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uplc::machine::cost_model::ExBudget;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PlutusScriptFile {
    pub r#type: String,
    pub description: String,
    pub cborHex: String,
}

impl PlutusScriptFile {
    pub fn new(script_type: &str, description: &str, cbor: &str) -> Self {
        PlutusScriptFile {
            r#type: script_type.to_string(),
            description: description.to_string(),
            cborHex: cbor.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlueprintFile {
    preamble: Preamble,
    validators: Vec<ValidatorBlueprint>,
}

impl BlueprintFile {
    pub fn get_validator(&self, title: &str) -> Option<ValidatorBlueprint> {
        self.validators.iter().find(|v| v.title == title).cloned()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Preamble {
    title: String,
    description: String,
    version: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidatorBlueprint {
    title: String,
    purpose: String,
    datum: Option<serde_json::Value>, // TODO: what is this type actually?
    redeemer: serde_json::Value,      // TODO: what is this type actually?
    compiledCode: String,
    hash: String,
}

impl ValidatorBlueprint {
    pub fn compiled_code(&self) -> String {
        self.compiledCode.clone()
    }
}

// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct DatumBlueprint {
//     title: String,
//     description: String,
//     anyOf: Vec<VariantBlueprint>,
// }
//
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RedeemerBlueprint {
//     title: String,
//     description: String,
//     anyOf: Vec<VariantBlueprint>,
// }
//
// #[allow(non_snake_case)]
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct VariantBlueprint {
//     dataType: String,
//     index: u32,
//     fields: serde_json::Value, // TODO: what is this type actually?
// }

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RawPlutusScriptError {
    #[error("Error in Aiken Apply: {0:?}")]
    AikenApply(String),
    #[error("Error in Aiken Eval: {error:?}, Logs: {logs:?}")]
    AikenEval { error: String, logs: Vec<String> },
    #[error("CML Error: {0:?}")]
    CMLError(String),
}

pub type RawPlutusScriptResult<T, E = RawPlutusScriptError> = Result<T, E>;

impl From<ExBudget> for ExecutionCost {
    fn from(value: ExBudget) -> Self {
        let mem = value.mem;
        let cpu = value.cpu;
        ExecutionCost { mem, cpu }
    }
}
