use naumachia::{
    output::Output as NauOutput,
    scripts::{
        plutus_minting_policy::OneParamPlutusPolicy,
        plutus_validator::plutus_data::{
            Constr,
            PlutusData,
        },
        raw_script::BlueprintFile,
        ScriptError,
        ScriptResult,
    },
};

const BLUEPRINT: &str = include_str!("../aiken/mint_nft/plutus.json");
const VALIDATOR_NAME: &str = "one_shot_nft.mint";

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
            constr: 0,
            fields: vec![PlutusData::BoundedBytes(tx_id_bytes)],
        });
        let output_index = PlutusData::BigInt((out_ref.output_index as i64).into()); // TODO: panic
        PlutusData::Constr(Constr {
            constr: 0,
            fields: vec![transaction_id, output_index],
        })
    }
}

pub fn get_parameterized_script(
) -> ScriptResult<OneParamPlutusPolicy<OutputReference, ()>> {
    let script_file: BlueprintFile = serde_json::from_str(BLUEPRINT)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let validator_blueprint = script_file.get_validator(VALIDATOR_NAME).ok_or(
        ScriptError::FailedToConstruct(format!(
            "Validator not listed in Blueprint: {:?}",
            VALIDATOR_NAME
        )),
    )?;
    let raw_script_validator = OneParamPlutusPolicy::from_blueprint(validator_blueprint)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::{
        output::Output,
        scripts::{
            context::{
                pub_key_hash_from_address_if_available,
                ContextBuilder,
            },
            MintingPolicy,
        },
        Address,
    };

    #[test]
    fn execute__succeeds_when_output_included() {
        let tx_hash = vec![1, 2, 3, 4];
        let index = 0;
        let owner = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let output: Output<()> =
            Output::new_wallet(tx_hash, index, owner.clone(), Default::default());

        let out_ref = OutputReference::from(&output);

        let param_script = get_parameterized_script().unwrap();
        let script = param_script.apply(out_ref).unwrap();

        let owner_pkh = pub_key_hash_from_address_if_available(&owner).unwrap();
        let ctx = ContextBuilder::new(owner_pkh)
            .add_specific_input(&output)
            .build_mint(&[]);
        let _eval = script.execute((), ctx).unwrap();
    }

    #[test]
    fn execute__fails_when_output_included() {
        let tx_hash = vec![1, 2, 3, 4];
        let index = 0;
        let owner = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let output: Output<()> =
            Output::new_wallet(tx_hash, index, owner.clone(), Default::default());

        let out_ref = OutputReference::from(&output);

        let param_script = get_parameterized_script().unwrap();
        let script = param_script.apply(out_ref).unwrap();

        let owner_pkh = pub_key_hash_from_address_if_available(&owner).unwrap();
        let ctx = ContextBuilder::new(owner_pkh).build_mint(&[]);
        let _eval = script.execute((), ctx).unwrap_err();
    }
}
