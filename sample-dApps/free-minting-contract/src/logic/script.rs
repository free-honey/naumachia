use naumachia::scripts::raw_policy_script::RawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::{ScriptError, ScriptResult};

// const SCRIPT_RAW: &str = include_str!("../../plutus/anyone-can-mint.plutus");
// const SCRIPT_RAW: &str = include_str!("../../plutus/free-minting.plutus");
const SCRIPT_RAW: &str = include_str!("../../plutus/free-minting-lite.plutus");

pub fn get_policy<R>() -> ScriptResult<RawPolicy<R>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPolicy::new_v1(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::address::Address;
    use naumachia::scripts::{ContextBuilder, MintingPolicy};

    #[test]
    fn can_execute_policy() {
        let owner = Address::new("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0");

        let script = get_policy().unwrap();

        let ctx = ContextBuilder::new(owner).build();

        let _eval = script.execute((), ctx).unwrap();
    }
}
