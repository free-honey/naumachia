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
    use naumachia::scripts::{MintingPolicy, ValidatorCode};

    #[test]
    fn different_salts_have_different_ids() {
        let param_script = checking_account_validator().unwrap();
        let nft_1 = SpendingTokenPolicy {
            inner: vec![1, 2, 3, 4, 5],
        };
        let script_1 = param_script.apply(nft_1).unwrap();
        let id_1 = script_1.script_hex().unwrap();
        let nft_2 = SpendingTokenPolicy {
            inner: vec![6, 7, 8, 9, 10],
        };
        let script_2 = param_script.apply(nft_2).unwrap();
        let id_2 = script_2.script_hex().unwrap();
        assert_ne!(id_1, id_2);
    }
}
