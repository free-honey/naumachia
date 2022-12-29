use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../time_locked/assets/time_lock/spend/payment_script.json");

pub fn get_script() -> ScriptResult<RawPlutusValidator<(), ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPlutusValidator::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::address::Address;
    use naumachia::scripts::{TxContext, ValidatorCode};

    #[test]
    fn test() {
        let script = get_script().unwrap();

        let owner = Address::new("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0");

        let ctx = TxContext { signer: owner };
        let cbor = script.script_hex().unwrap();
        dbg!(&cbor);
        let _eval = script.execute((), (), ctx).unwrap();
    }
}
