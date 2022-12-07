use serde::{Deserialize, Serialize};
use thiserror::Error;

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
