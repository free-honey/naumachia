use naumachia::scripts::raw_policy_script::OneParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../mint_nft/assets/one_shot_nft/mint/payment_script.json");

pub struct Input;

pub fn get_parameterized_script() -> ScriptResult<OneParamRawPolicy<Input, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = OneParamRawPolicy::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
