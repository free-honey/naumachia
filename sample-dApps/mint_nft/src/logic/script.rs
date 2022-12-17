use naumachia::output::Output as NauOutput;
use naumachia::scripts::raw_policy_script::OneParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../mint_nft/assets/one_shot_nft/mint/payment_script.json");

// pub type OutputReference {
//   transction_id: TransactionId,
//   output_index: Int,
// }
pub struct OutputReference {
    pub transaction_id: TransactionId,
    pub output_index: u64,
}

// pub type TransactionId {
//   hash: Hash(Transaction),
// }
pub type TransactionId = Vec<u8>;

impl<T> From<&NauOutput<T>> for OutputReference {
    fn from(output: &NauOutput<T>) -> Self {
        let id = output.id();
        let transaction_id = id.tx_hash().bytes().collect();
        let output_index = id.index();
        OutputReference {
            transaction_id,
            output_index,
        }
    }
}

impl From<OutputReference> for PlutusData {
    fn from(_: OutputReference) -> Self {
        todo!()
    }
}

pub fn get_parameterized_script() -> ScriptResult<OneParamRawPolicy<OutputReference, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = OneParamRawPolicy::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
