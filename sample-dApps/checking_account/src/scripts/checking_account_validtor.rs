use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::raw_validator_script::OneParamRawValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../checking/assets/checking_account_validator/spend/payment_script.json");

pub struct SpendingTokenPolicy {
    inner: Vec<u8>,
}

impl From<SpendingTokenPolicy> for PlutusData {
    fn from(value: SpendingTokenPolicy) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub fn checking_account_validator(
) -> ScriptResult<OneParamRawValidator<SpendingTokenPolicy, (), ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = OneParamRawValidator::new_v2(script_file)
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
        let signer = Address::new("6082e016828989cd9d809b50d6976d9efa9bc5b2c1a78d4b3bfa1bb83b");
        let param_script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let spending_token = SpendingTokenPolicy {
            inner: policy.clone(),
        };
        let ctx = ContextBuilder::new(signer.clone())
            .build_input(
                "73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b",
                0,
                signer.to_str(),
            )
            .with_value(&hex::encode(&policy), "", 1)
            .finish_input()
            .build();

        let script = param_script.apply(spending_token).unwrap();

        let _eval = script.execute((), (), ctx).unwrap();
    }

    #[test]
    fn fails_if_not_spending_token_in_inputs() {
        let signer = Address::new("6082e016828989cd9d809b50d6976d9efa9bc5b2c1a78d4b3bfa1bb83b");
        let param_script = checking_account_validator().unwrap();
        let policy = vec![1, 2, 3, 4, 5];
        let spending_token = SpendingTokenPolicy {
            inner: policy.clone(),
        };
        let ctx = ContextBuilder::new(signer.clone())
            .build_input(
                "73d65e0b9b68ebf3971b6ccddc75900dd62f9845f5ab972e469c5d803973015b",
                0,
                signer.to_str(),
            )
            .finish_input()
            .build();

        let script = param_script.apply(spending_token).unwrap();

        let _eval = script.execute((), (), ctx).unwrap_err();
    }
}
