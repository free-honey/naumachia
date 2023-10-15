use crate::scripts::ExecutionCost;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uplc::machine::cost_model::ExBudget;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Serialize, Deserialize, Debug)]
/// Representation of a standard Plutus Script file
pub struct PlutusScriptFile {
    /// Type of the script
    pub r#type: String,
    /// Description of the script
    pub description: String,
    /// Raw CBOR script bytes
    pub cborHex: String,
}

impl PlutusScriptFile {
    /// Constructor for a PlutusScriptFile
    pub fn new(script_type: &str, description: &str, cbor: &str) -> Self {
        PlutusScriptFile {
            r#type: script_type.to_string(),
            description: description.to_string(),
            cborHex: cbor.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// Representation of a CIP-0057 Blueprint file
pub struct BlueprintFile {
    preamble: Preamble,
    validators: Vec<ValidatorBlueprint>,
}

impl BlueprintFile {
    /// Get a specific validator from the Blueprint file representation
    pub fn get_validator(&self, title: &str) -> Option<ValidatorBlueprint> {
        self.validators.iter().find(|v| v.title == title).cloned()
    }
}

/// Preable of a CIP-0057 Blueprint file
#[derive(Serialize, Deserialize, Debug)]
pub struct Preamble {
    title: String,
    description: String,
    version: String,
}

/// Representation of a CIP-0057 Validator Blueprint
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidatorBlueprint {
    title: String,
    datum: Option<serde_json::Value>, // TODO: what is this type actually?
    redeemer: serde_json::Value,      // TODO: what is this type actually?
    compiledCode: String,
    hash: String,
}

impl ValidatorBlueprint {
    /// Get the hex bytes of the compiled Plutus script
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

/// Error from dealing with Plutus Scripts
#[allow(missing_docs)]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlutusScriptError {
    #[error("Error in Aiken Apply: {0:?}")]
    AikenApply(String),
    #[error("Error in Aiken Eval: {error:?}, Logs: {logs:?}")]
    AikenEval { error: String, logs: Vec<String> },
    #[error("CML Error: {0:?}")]
    CMLError(String),
}

#[allow(missing_docs)]
pub type RawPlutusScriptResult<T, E = PlutusScriptError> = Result<T, E>;

impl From<ExBudget> for ExecutionCost {
    fn from(value: ExBudget) -> Self {
        let mem = value.mem;
        let cpu = value.cpu;
        ExecutionCost { mem, cpu }
    }
}
