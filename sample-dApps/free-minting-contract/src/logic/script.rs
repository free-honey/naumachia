use cardano_multiplatform_lib::{
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use naumachia::scripts::raw_policy_script::RawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::{
    address::Address,
    scripts::{ScriptError, ScriptResult, TxContext, ValidatorCode},
};

// const SCRIPT_RAW: &str = include_str!("../../plutus/anyone-can-mint.plutus");
const SCRIPT_RAW: &str = include_str!("../../plutus/free-minting.plutus");

pub fn get_policy() -> ScriptResult<RawPolicy> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPolicy::new_v1(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
