use crate::datum::CheckingAccountDatums;
use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::RawPlutusValidator;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str = include_str!("../../checking/plutus.json");
const VALIDATOR_NAME: &str = "pull_validator.spend";

pub fn pull_validator() -> ScriptResult<RawPlutusValidator<CheckingAccountDatums, ()>> {
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
    use crate::datum::{AllowedPuller, CheckingAccount, CheckingAccountDatums};
    use crate::Address;
    use naumachia::scripts::context::{
        pub_key_hash_from_address_if_available, ContextBuilder, PubKeyHash, TxContext,
    };
    use naumachia::scripts::ValidatorCode;
    use naumachia::Network;

    struct PullTestContext {
        pub signer_pkh: PubKeyHash,
        pub range_lower: Option<(i64, bool)>,
        pub range_upper: Option<(i64, bool)>,

        pub input_address: Address,
        pub input_tx_id: Vec<u8>,
        pub input_index: u64,
        pub input_token_policy_id: String,
        pub input_datum: Option<CheckingAccountDatums>,

        pub output_address: Address,
        pub output_token_policy_id: String,
        pub output_datum: Option<CheckingAccountDatums>,

        pub account_input_address: Address,
        pub account_input_tx_id: Vec<u8>,
        pub account_input_index: u64,
        pub account_input_nft_id: String,
        pub account_input_token_amt: u64,
        pub account_input_ada: u64,
        pub account_input_datum: Option<CheckingAccountDatums>,

        pub account_output_address: Address,
        pub account_output_nft_id: String,
        pub account_output_token_amt: u64,
        pub account_output_ada: u64,
        pub account_output_datum: Option<CheckingAccountDatums>,
    }

    impl PullTestContext {
        pub fn pull_happy_path() -> Self {
            let account_owner = Address::from_bech32(
                "addr_test1vz3ppzmmzuz0nlsjeyrqjm4pvdxl3cyfe8x06eg6htj2gwgv02qjt",
            )
            .unwrap();
            let account_owner_pubkey_hash =
                pub_key_hash_from_address_if_available(&account_owner).unwrap();

            let puller = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
            let signer_pubkey_hash = pub_key_hash_from_address_if_available(&puller).unwrap();
            let checking_account_address = Address::from_bech32(
                "addr_test1wpe9mt7mkjmkkuqjmevzafm6mle9t0spprr9335q0e6p92cur7fvl",
            )
            .unwrap();
            let checking_account_nft_id = [7, 7, 7, 7, 7];
            let signer_pkh = pub_key_hash_from_address_if_available(&puller).unwrap();
            let script = pull_validator().unwrap();
            let input_tx_id = [8, 8, 8, 8];
            let account_input_tx_id = [9, 8, 7, 6];
            let input_tx_index = 0;
            let account_input_tx_index = 0;
            let network = Network::Testnet;
            let script_address = script.address(network).unwrap();
            let spending_token = vec![5, 5, 5, 5];
            let original_balance = 10_000;
            let pull_amount = 10;
            let input_datum = AllowedPuller {
                owner: account_owner_pubkey_hash.clone(),
                puller: signer_pubkey_hash.clone(),
                amount_lovelace: pull_amount,
                next_pull: 10,
                period: 10,
                spending_token: spending_token.clone(),
                checking_account_nft: checking_account_nft_id.to_vec(),
            }
            .into();
            let policy_id = hex::encode(&spending_token);
            let nft_id = hex::encode(&checking_account_nft_id);
            let output_datum = AllowedPuller {
                owner: account_owner_pubkey_hash.clone(),
                puller: signer_pubkey_hash,
                amount_lovelace: pull_amount,
                next_pull: 20,
                period: 10,
                spending_token: spending_token.clone(),
                checking_account_nft: checking_account_nft_id.to_vec(),
            }
            .into();

            let account_datum: CheckingAccountDatums = CheckingAccount {
                owner: account_owner_pubkey_hash,
                spend_token_policy: spending_token,
            }
            .into();
            PullTestContext {
                signer_pkh,
                range_lower: Some((11, true)),
                range_upper: None,
                input_address: script_address.clone(),
                input_tx_id: input_tx_id.to_vec(),
                input_index: input_tx_index,
                input_token_policy_id: policy_id.clone(),
                input_datum: Some(input_datum),
                output_address: script_address,
                output_token_policy_id: policy_id.clone(),
                output_datum: Some(output_datum),
                account_input_address: checking_account_address.clone(),
                account_input_tx_id: account_input_tx_id.to_vec(),
                account_input_index: account_input_tx_index,
                account_input_nft_id: nft_id.clone(),
                account_input_token_amt: 1,
                account_input_ada: original_balance,
                account_input_datum: Some(account_datum.clone()),
                account_output_address: checking_account_address,
                account_output_nft_id: nft_id.clone(),
                account_output_token_amt: 1,
                account_output_ada: original_balance - pull_amount,
                account_output_datum: Some(account_datum),
            }
        }

        pub fn build(&self) -> TxContext {
            let mut input_builder = ContextBuilder::new(self.signer_pkh.clone())
                .with_range(self.range_lower.clone(), self.range_upper.clone())
                .with_input(&self.input_tx_id, self.input_index, &self.input_address)
                .with_value(&self.input_token_policy_id, "something", 1);
            if let Some(input_datum) = &self.input_datum {
                input_builder = input_builder.with_inline_datum(input_datum.clone())
            }
            let mut output_builder = input_builder
                .finish_input()
                .with_output(&self.output_address)
                .with_value(&self.output_token_policy_id, "something", 1);
            if let Some(output_datum) = &self.output_datum {
                output_builder = output_builder.with_inline_datum(output_datum.clone())
            }
            let mut input_builder = output_builder
                .finish_output()
                .with_input(
                    &self.account_input_tx_id,
                    self.account_input_index,
                    &self.account_input_address,
                )
                .with_value(
                    &self.account_input_nft_id,
                    "nft",
                    self.account_input_token_amt,
                )
                .with_value("", "", self.account_input_ada);
            if let Some(input_datum) = &self.account_input_datum {
                input_builder = input_builder.with_inline_datum(input_datum.to_owned())
            }
            let mut output_builder = input_builder
                .finish_input()
                .with_output(&self.account_output_address)
                .with_value(
                    &self.account_output_nft_id,
                    "nft",
                    self.account_output_token_amt,
                )
                .with_value("", "", self.account_output_ada);
            if let Some(output_datum) = &self.account_output_datum {
                output_builder = output_builder.with_inline_datum(output_datum.to_owned())
            }
            output_builder
                .finish_output()
                .build_spend(&self.input_tx_id, self.input_index)
        }
    }

    #[test]
    fn execute__pull_happy_path() {
        let ctx_builder = PullTestContext::pull_happy_path();
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let script = pull_validator().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__wrong_puller_fails() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();
        let not_puller = Address::from_bech32("addr_test1qqjc95k0kd3apsk0akfc8dvpr72hsv6sc9vnyl5kxs9rsl3kfufq25ftkfjfceqnq4lezek9fth36mwt9m934h95j6ysepjhrc").unwrap();
        let not_puller_pkh = pub_key_hash_from_address_if_available(&not_puller).unwrap();

        // when
        ctx_builder.signer_pkh = not_puller_pkh;

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__before_next_pull_date_fails() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.range_lower = Some((8, true));

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_not_inclusive_fails() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.range_lower = Some((10, false));

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__same_date_inclusive_succeeds() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.range_lower = Some((10, true));

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__no_new_pull_datum_fails() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.output_datum = None;

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_next_pull_wrong() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => {
                let AllowedPuller { next_pull, .. } = old_allowed_puller;
                AllowedPuller {
                    next_pull: next_pull - 1,
                    ..old_allowed_puller
                }
                .into()
            }
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_period_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => {
                let AllowedPuller { period, .. } = old_allowed_puller;
                AllowedPuller {
                    period: period + 1,
                    ..old_allowed_puller
                }
                .into()
            }
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_spending_token_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();
        let bad_spending_token = vec![6, 6, 6, 6];

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => AllowedPuller {
                spending_token: bad_spending_token,
                ..old_allowed_puller
            }
            .into(),
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_account_nft_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        let bad_nft_id = vec![5, 4, 3, 4, 1, 6, 2];
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => AllowedPuller {
                checking_account_nft: bad_nft_id,
                ..old_allowed_puller
            }
            .into(),
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_pull_amount_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => {
                let AllowedPuller {
                    amount_lovelace, ..
                } = old_allowed_puller;
                AllowedPuller {
                    amount_lovelace: amount_lovelace + 1, // Change pull amount
                    ..old_allowed_puller
                }
                .into()
            }
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_puller_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();
        let wrong_puller =
            Address::from_bech32("addr_test1vzpwq95z3xyum8vqndgdd9mdnmafh3djcxnc6jemlgdmswcve6tkw")
                .unwrap();
        let wrong_puller_pubkey_hash =
            pub_key_hash_from_address_if_available(&wrong_puller).unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller(old_allowed_puller) => AllowedPuller {
                puller: wrong_puller_pubkey_hash,
                ..old_allowed_puller
            }
            .into(),
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_output_does_not_include_spending_token() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.output_token_policy_id = "".to_string(); // Replace spending token with lovelace

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_account_nft_not_replaced() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        let wrong_address =
            Address::from_bech32("addr_test1vz3ppzmmzuz0nlsjeyrqjm4pvdxl3cyfe8x06eg6htj2gwgv02qjt")
                .unwrap();
        ctx_builder.account_output_address = wrong_address; // Replace spending token with lovelace

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_too_much_is_pulled() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.account_output_ada = ctx_builder.account_output_ada - 100; // pull too much

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_account_datum_not_replaced() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.account_output_datum = None;

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_account_datum_owner_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();
        let wrong_owner =
            Address::from_bech32("addr_test1wr34avr87aq3aj0xlgj78jqjwjfppcj2ctsz8rppr2w8upc4jayvq")
                .unwrap();
        let wrong_owner_pubkey_hash = pub_key_hash_from_address_if_available(&wrong_owner).unwrap();

        // when
        let new_datum = match ctx_builder.account_output_datum.unwrap() {
            CheckingAccountDatums::CheckingAccount(old_checking_account) => CheckingAccount {
                owner: wrong_owner_pubkey_hash,
                ..old_checking_account
            }
            .into(),
            _ => panic!("wrong variant"),
        };
        ctx_builder.account_output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__fails_if_account_datum_spend_token_changes() {
        // given
        let mut ctx_builder = PullTestContext::pull_happy_path();
        let script = pull_validator().unwrap();
        let bad_token_id = vec![2, 3, 4, 5, 2, 6, 25, 2, 4];

        // when
        let new_datum = match ctx_builder.account_output_datum.unwrap() {
            CheckingAccountDatums::CheckingAccount(old_checking_account) => CheckingAccount {
                spend_token_policy: bad_token_id,
                ..old_checking_account
            }
            .into(),
            _ => panic!("wrong variant"),
        };
        ctx_builder.account_output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__remove_happy_path() {
        let account_address =
            Address::from_bech32("addr_test1vz3ppzmmzuz0nlsjeyrqjm4pvdxl3cyfe8x06eg6htj2gwgv02qjt")
                .unwrap();
        let puller_address = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let owner = pub_key_hash_from_address_if_available(&account_address).unwrap();
        let puller = pub_key_hash_from_address_if_available(&puller_address).unwrap();
        let spending_token = vec![5, 5, 5, 5];
        let checking_account_nft = vec![7, 7, 7, 7, 7];

        let datum = AllowedPuller {
            owner: owner.clone(),
            puller,
            amount_lovelace: 100,
            next_pull: 9999999,
            period: 234,
            spending_token,
            checking_account_nft,
        }
        .into();
        let tx_id = [9, 8, 7, 6];
        let tx_index = 0;

        let script = pull_validator().unwrap();
        let ctx = ContextBuilder::new(owner).build_spend(&tx_id, tx_index);

        let _eval = script.execute(datum, (), ctx).unwrap();
    }
}
