use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::RawPlutusValidator;
use crate::scripts::{MintingPolicy, ScriptResult, TxContext};
use crate::PolicyId;
use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script};
use std::marker::PhantomData;

pub struct RawPolicy {
    script_file: PlutusScriptFile,
    cml_script: PlutusScript,
}

impl RawPolicy {
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_policy = RawPolicy {
            script_file,
            cml_script,
        };
        Ok(v1_policy)
    }
}

impl MintingPolicy for RawPolicy {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()> {
        todo!()
    }

    fn id(&self) -> PolicyId {
        let script_hash = self.cml_script.hash();
        let policy_id = PolicyId::NativeToken(script_hash.to_string(), None);
        policy_id
    }
}
