use crate::CheckingAccountDatums;
use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const BLUEPRINT: &str = include_str!("../../checking/plutus.json");
const VALIDATOR_NAME: &str = "checking_account_validator";

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
    let script_file: BlueprintFile = serde_json::from_str(BLUEPRINT)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let validator_blueprint =
        script_file
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
    use hex;
    use naumachia::address::Address;
    use naumachia::scripts::context::ContextBuilder;
    use naumachia::scripts::ValidatorCode;

    #[test]
    fn succeeds_if_spending_token_in_inputs() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let ctx = ContextBuilder::new(signer.clone())
            .build_input(
                &hex::decode("73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b")
                    .unwrap(),
                0,
                &signer.bytes().unwrap(),
            )
            .with_value(&hex::encode(&policy), "", 1)
            .finish_input()
            .build();

        let owner = Address::new("addr_test1vqm77xl444msdszx9s982zu95hh03ztw4rsp8xcs2ty3xucr40ujs");
        let datum = CheckingAccountDatums::CheckingAccount {
            owner,
            spend_token_policy: hex::encode(policy),
        };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn fails_if_not_spending_token_in_inputs() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let ctx = ContextBuilder::new(signer.clone())
            .build_input(
                &hex::decode("73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b")
                    .unwrap(),
                0,
                &signer.bytes().unwrap(),
            )
            .finish_input()
            .build();

        let owner = Address::new("addr_test1vqm77xl444msdszx9s982zu95hh03ztw4rsp8xcs2ty3xucr40ujs");
        let datum = CheckingAccountDatums::CheckingAccount {
            owner,
            spend_token_policy: hex::encode(policy),
        };

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }
}
