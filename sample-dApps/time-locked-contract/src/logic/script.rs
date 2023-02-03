use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::plutus_data::{BigInt, Constr, PlutusData};
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const BLUEPRINT: &str = include_str!("../../time_locked/plutus.json");
const VALIDATOR_NAME: &str = "time_lock";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Timestamp {
    pub milliseconds: i64,
}

impl Timestamp {
    pub fn new(milliseconds: i64) -> Self {
        Timestamp { milliseconds }
    }
}

impl From<Timestamp> for PlutusData {
    fn from(value: Timestamp) -> Self {
        let num = value.milliseconds;
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        let milliseconds = PlutusData::BigInt(BigInt::Int { neg, val });
        let constr = Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![milliseconds],
        };
        PlutusData::Constr(constr)
    }
}

impl TryFrom<PlutusData> for Timestamp {
    type Error = String;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(mut constr) => {
                let field = constr
                    .fields
                    .pop()
                    .ok_or(format!("Data constr fields empty"))?;
                match field {
                    PlutusData::BigInt(big_int) => {
                        let milliseconds = big_int.into();
                        let timestamp = Timestamp { milliseconds };
                        Ok(timestamp)
                    }
                    _ => Err(format!(
                        "Expected BigInt as first constr field, found: {:?}",
                        field
                    )),
                }
            }
            _ => Err(format!("Expected Constr PlutusData, found: {:?}", value)),
        }
    }
}

pub fn get_script() -> ScriptResult<RawPlutusValidator<i64, ()>> {
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
    use naumachia::scripts::context::{pub_key_has_from_address_if_available, ContextBuilder};
    use naumachia::scripts::ValidatorCode;
    use naumachia::Address;

    #[test]
    fn test_in_range_succeeds() {
        let script = get_script().unwrap();

        let owner = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();

        let owner_pkh = pub_key_has_from_address_if_available(&owner).unwrap();
        let ctx = ContextBuilder::new(owner_pkh)
            .with_range(Some((80, true)), None)
            .build_spend(&vec![], 0);

        let datum = 69_i64;
        script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn test_out_of_range_fails() {
        let script = get_script().unwrap();

        let owner = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();

        let owner_pkh = pub_key_has_from_address_if_available(&owner).unwrap();
        let ctx = ContextBuilder::new(owner_pkh)
            .with_range(Some((10, true)), None)
            .build_spend(&vec![], 0);

        let datum = 69_i64;
        let error = script.execute(datum, (), ctx);

        assert!(error.is_err());
    }
}
