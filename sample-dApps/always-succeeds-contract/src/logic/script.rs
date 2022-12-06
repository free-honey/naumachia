use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str = include_str!("../../plutus/always-succeeds-spending.plutus");

pub fn get_script() -> ScriptResult<RawPlutusValidator<(), ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPlutusValidator::new_v1(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
