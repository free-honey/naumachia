use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const BLUEPRINT: &str = include_str!("../../always_succeeds/plutus.json");
const VALIDATOR_NAME: &str = "always_true";

pub fn get_script() -> ScriptResult<RawPlutusValidator<(), ()>> {
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
    use naumachia::address::Address;
    use naumachia::scripts::context::ContextBuilder;
    use naumachia::scripts::ValidatorCode;

    #[test]
    fn test() {
        let script = get_script().unwrap();

        let owner = Address::new("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0");

        let ctx = ContextBuilder::new(owner).build();
        script.execute((), (), ctx).unwrap();
    }
}
