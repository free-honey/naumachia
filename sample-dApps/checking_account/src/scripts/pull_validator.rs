use crate::CheckingAccountDatums;
use naumachia::scripts::raw_script::{BlueprintFile, PlutusScriptFile};
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str = include_str!("../../checking/plutus.json");
const VALIDATOR_NAME: &str = "pull_validator";

pub fn spend_token_policy() -> ScriptResult<RawPlutusValidator<CheckingAccountDatums, ()>> {
    let blueprint: BlueprintFile = serde_json::from_str(SCRIPT_RAW)
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

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, CheckingAccountDatums};
    use naumachia::address::PolicyId;
    use naumachia::scripts::context::{pub_key_hash_from_address_if_available, ContextBuilder};
    use naumachia::scripts::ValidatorCode;

    #[test]
    fn execute__after_next_pull_date_succeeds() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let script = spend_token_policy().unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .build_spend(&[8, 8, 8, 8], 0);

        let spending_token_policy = vec![1, 2, 3, 4];
        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__before_next_pull_date_fails() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let script = spend_token_policy().unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((8, true)), None)
            .build_spend(&[8, 8, 8, 8], 0);

        let spending_token_policy = vec![1, 2, 3, 4];
        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_not_inclusive_fails() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let script = spend_token_policy().unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((10, false)), None)
            .build_spend(&[8, 8, 8, 8], 0);

        let spending_token_policy = vec![1, 2, 3, 4];
        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_inclusive_succeeds() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let script = spend_token_policy().unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((10, true)), None)
            .build_spend(&[8, 8, 8, 8], 0);

        let spending_token_policy = vec![1, 2, 3, 4];
        let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    // #[test]
    // fn execute__unlock_one_spendtoken_succeeds() {
    //     let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
    //     let script = spend_token_policy().unwrap();
    //     let ctx = ContextBuilder::new(signer.clone())
    //         .with_range(Some((11, true)), None)
    //         .build();
    //
    //     let spending_token_policy = vec![1, 2, 3, 4];
    //     let datum = CheckingAccountDatums::AllowedPuller { next_pull: 10 };
    //
    //     let _eval = script.execute(datum, (), ctx).unwrap();
    // }
}
