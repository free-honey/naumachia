use crate::CheckingAccountDatums;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../checking/assets/pull_validator/spend/payment_script.json");

pub fn spend_token_policy() -> ScriptResult<RawPlutusValidator<CheckingAccountDatums, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPlutusValidator::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::CheckingAccountDatums;
    use naumachia::address::Address;
    use naumachia::scripts::context::ContextBuilder;
    use naumachia::scripts::ValidatorCode;

    #[test]
    fn execute__after_next_pull_date_succeeds() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = spend_token_policy().unwrap();
        let ctx = ContextBuilder::new(signer.clone())
            .with_range(Some((11, true)), None)
            .build();

        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__before_next_pull_date_fails() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = spend_token_policy().unwrap();
        let ctx = ContextBuilder::new(signer.clone())
            .with_range(Some((8, true)), None)
            .build();

        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_not_inclusive_fails() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = spend_token_policy().unwrap();
        let ctx = ContextBuilder::new(signer.clone())
            .with_range(Some((10, false)), None)
            .build();

        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_inclusive_succeeds() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let script = spend_token_policy().unwrap();
        let ctx = ContextBuilder::new(signer.clone())
            .with_range(Some((10, true)), None)
            .build();

        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }
}
