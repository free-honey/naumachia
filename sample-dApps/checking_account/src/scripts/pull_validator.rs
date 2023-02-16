use crate::CheckingAccountDatums;
use naumachia::scripts::raw_script::BlueprintFile;
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
    use naumachia::scripts::context::{pub_key_hash_from_address_if_available, ContextBuilder};
    use naumachia::scripts::ValidatorCode;

    const NETWORK: u8 = 0;

    #[test]
    fn execute__after_next_pull_date_succeeds() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let script = spend_token_policy().unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .build_spend(&[8, 8, 8, 8], 0);
        let spending_token = vec![5, 5, 5, 5];

        let datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 0,
            spending_token,
        };

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

        let spending_token = vec![5, 5, 5, 5];
        let datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 0,
            spending_token,
        };

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
        let spending_token = vec![5, 5, 5, 5];

        let datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 0,
            spending_token,
        };

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
        let spending_token = vec![5, 5, 5, 5];

        let datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 0,
            spending_token,
        };

        let _eval = script.execute(datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__happy_path() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token: spending_token.clone(),
        };
        let policy_id = hex::encode(&spending_token);
        let output_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 10,
            spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(output_datum)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__no_new_pull_datum_fails() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let policy_id = hex::encode(&spending_token);
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 0,
            spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_value(&policy_id, "something", 1)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_next_pull_wrong() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let policy_id = hex::encode(&spending_token);
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token: spending_token.clone(),
        };
        let output_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(output_datum)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_period_changes() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let policy_id = hex::encode(&spending_token);
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token: spending_token.clone(),
        };
        let output_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 0,
            spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(output_datum)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_spending_token_changes() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let policy_id = hex::encode(&spending_token);
        let bad_spending_token = vec![6, 6, 6, 6];
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token,
        };
        let output_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 10,
            spending_token: bad_spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(output_datum)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_output_does_not_include_spending_token() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
        let script = spend_token_policy().unwrap();
        let input_tx_id = [8, 8, 8, 8];
        let input_tx_index = 0;
        let script_address = script.address(NETWORK).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let input_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 0,
            period: 10,
            spending_token: spending_token.clone(),
        };
        let policy_id = hex::encode(&spending_token);
        let output_datum = CheckingAccountDatums::AllowedPuller {
            next_pull: 10,
            period: 10,
            spending_token,
        };
        let ctx = ContextBuilder::new(signer_pkh)
            .with_range(Some((11, true)), None)
            .with_input(&input_tx_id, input_tx_index, &script_address)
            .with_value(&policy_id, "something", 1)
            .with_inline_datum(input_datum.clone())
            .finish_input()
            .with_output(&script_address)
            .with_inline_datum(output_datum)
            .finish_output()
            .build_spend(&input_tx_id, input_tx_index);

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }
}
