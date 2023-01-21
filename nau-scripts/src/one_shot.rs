use naumachia::{
    output::Output as NauOutput,
    scripts::{
        raw_policy_script::OneParamRawPolicy,
        raw_script::PlutusScriptFile,
        raw_validator_script::plutus_data::{Constr, PlutusData},
        ScriptError, ScriptResult,
    },
};

const SCRIPT_RAW: &str =
    include_str!("../aiken/mint_nft/assets/one_shot_nft/mint/payment_script.json");

// pub type OutputReference {
//   transction_id: TransactionId,
//   output_index: Int,
// }
// TODO: Move to context
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
        let transaction_id = id.tx_hash().to_vec();
        let output_index = id.index();
        OutputReference {
            transaction_id,
            output_index,
        }
    }
}

impl From<OutputReference> for PlutusData {
    fn from(out_ref: OutputReference) -> Self {
        let tx_id_bytes = out_ref.transaction_id;
        let transaction_id = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(tx_id_bytes)],
        });
        let output_index = PlutusData::BigInt((out_ref.output_index as i64).into()); // TODO: panic
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![transaction_id, output_index],
        })
    }
}

pub fn get_parameterized_script() -> ScriptResult<OneParamRawPolicy<OutputReference, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = OneParamRawPolicy::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::address::Address;
    use naumachia::output::{Output, OutputId};
    use naumachia::scripts::context::ContextBuilder;
    use naumachia::scripts::MintingPolicy;

    #[test]
    fn execute__succeeds_when_output_included() {
        let id = OutputId::new(vec![1, 2, 3, 4], 0);
        let owner = Address::new("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0");
        let output = Output::<()>::Wallet {
            id,
            owner: owner.clone(),
            values: Default::default(),
        };

        let out_ref = OutputReference::from(&output);

        let param_script = get_parameterized_script().unwrap();
        let script = param_script.apply(out_ref).unwrap();

        let ctx = ContextBuilder::new(owner)
            .add_specific_input(&output)
            .build();
        let _eval = script.execute((), ctx).unwrap();
    }

    #[test]
    fn execute__fails_when_output_included() {
        let id = OutputId::new(vec![1, 2, 3, 4], 0);
        let owner = Address::new("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0");
        let output = Output::<()>::Wallet {
            id,
            owner: owner.clone(),
            values: Default::default(),
        };

        let out_ref = OutputReference::from(&output);

        let param_script = get_parameterized_script().unwrap();
        let script = param_script.apply(out_ref).unwrap();

        let ctx = ContextBuilder::new(owner).build();
        let _eval = script.execute((), ctx).unwrap_err();
    }
}
