use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_CBOR: &str = include_str!("../../always_succeeds/assets/always_true/spend/script.txt");

pub fn get_script() -> ScriptResult<RawPlutusValidator<(), ()>> {
    let script_file = PlutusScriptFile::new("AikenV2", "should always succeed", SCRIPT_CBOR);
    let raw_script_validator = RawPlutusValidator::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
