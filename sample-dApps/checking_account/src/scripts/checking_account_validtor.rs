use crate::datum::CheckingAccountDatums;
use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const BLUEPRINT: &str = include_str!("../../checking/plutus.json");
const VALIDATOR_NAME: &str = "checking_account_validator.spend";

pub struct SpendingTokenPolicy {
    inner: Vec<u8>,
}

impl From<Vec<u8>> for SpendingTokenPolicy {
    fn from(value: Vec<u8>) -> Self {
        SpendingTokenPolicy { inner: value }
    }
}

impl From<SpendingTokenPolicy> for PlutusData {
    fn from(value: SpendingTokenPolicy) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub fn checking_account_validator() -> ScriptResult<RawPlutusValidator<CheckingAccountDatums, ()>> {
    let blueprint: BlueprintFile = serde_json::from_str(BLUEPRINT)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let validator_blueprint =
        blueprint
            .get_validator(VALIDATOR_NAME)
            .ok_or(ScriptError::FailedToConstruct(format!(
                "Validator not listed in Blueprint: {:?}",
                VALIDATOR_NAME
            )))?;
    let raw_script_validator = RawPlutusValidator::from_blueprint(validator_blueprint)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datum::CheckingAccount;
    use hex;
    use naumachia::scripts::context::{pub_key_hash_from_address_if_available, ContextBuilder};
    use naumachia::scripts::ValidatorCode;
    use naumachia::Address;

    #[test]
    fn succeeds_if_spending_token_in_inputs() {
        let signer = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_input(
                &hex::decode("73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b")
                    .unwrap(),
                0,
                &signer,
            )
            .with_value(&hex::encode(&policy), "", 1)
            .finish_input()
            .build_spend(&vec![], 0);

        let owner =
            Address::from_bech32("addr_test1vqm77xl444msdszx9s982zu95hh03ztw4rsp8xcs2ty3xucr40ujs")
                .unwrap();
        let owner_pubkey_hash = pub_key_hash_from_address_if_available(&owner).unwrap();
        let datum = CheckingAccount {
            owner: owner_pubkey_hash,
            spend_token_policy: policy,
        }
        .into();

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn fails_if_not_spending_token_in_inputs() {
        let signer = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_input(
                &hex::decode("73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b")
                    .unwrap(),
                0,
                &signer,
            )
            .finish_input()
            .build_spend(&vec![], 0);

        let owner =
            Address::from_bech32("addr_test1vqm77xl444msdszx9s982zu95hh03ztw4rsp8xcs2ty3xucr40ujs")
                .unwrap();
        let owner_pubkey_hash = pub_key_hash_from_address_if_available(&owner).unwrap();
        let datum = CheckingAccount {
            owner: owner_pubkey_hash,
            spend_token_policy: policy,
        }
        .into();

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }
}
