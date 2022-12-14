use naumachia::scripts::raw_policy_script::RawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::{ScriptError, ScriptResult};

// const SCRIPT_RAW: &str = include_str!("../../plutus/anyone-can-mint.plutus");
// const SCRIPT_RAW: &str = include_str!("../../plutus/free-minting.plutus");
const SCRIPT_RAW: &str = include_str!("../../plutus/free-minting-lite.plutus");

pub fn get_policy<R>() -> ScriptResult<RawPolicy<R>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPolicy::new_v1(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
