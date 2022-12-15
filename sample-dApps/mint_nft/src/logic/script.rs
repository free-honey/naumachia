use naumachia::output::Output as NauOutput;
use naumachia::scripts::raw_policy_script::OneParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../mint_nft/assets/one_shot_nft/mint/payment_script.json");

// pub type Input {
//   output_reference: OutputReference,
//   output: Output,
// }
pub struct Input {
    output_reference: OutputReference,
    output: Output,
}

// pub type OutputReference {
//   transction_id: TransactionId,
//   output_index: Int,
// }
pub struct OutputReference {
    transaction_id: TransactionId,
    output_index: u32,
}

// pub type Output {
//   address: Address,
//   value: Value,
//   datum: DatumOption,
//   reference_script: Option(ScriptHash),
// }
pub struct Output {}

// pub type TransactionId {
//   hash: Hash(Transaction),
// }
pub struct TransactionId {}

impl<T> From<NauOutput<T>> for Input {
    fn from(_: NauOutput<T>) -> Self {
        todo!()
    }
}

impl From<Input> for PlutusData {
    fn from(_: Input) -> Self {
        todo!()
    }
}

pub fn get_parameterized_script() -> ScriptResult<OneParamRawPolicy<Input, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = OneParamRawPolicy::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}
