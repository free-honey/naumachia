use crate::CheckingAccountDatums;
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
    use crate::{Address, CheckingAccountDatums};
    use naumachia::scripts::context::{
        pub_key_hash_from_address_if_available, ContextBuilder, PubKeyHash, TxContext,
    };
    use naumachia::scripts::ValidatorCode;

    const NETWORK: u8 = 0;

    struct TestContext {
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

        pub account_output_address: Address,
        pub account_output_nft_id: String,
        pub account_output_token_amt: u64,
    }

    impl TestContext {
        pub fn happy_path() -> Self {
            let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
            let checking_account_address = Address::from_bech32(
                "addr_test1wpe9mt7mkjmkkuqjmevzafm6mle9t0spprr9335q0e6p92cur7fvl",
            )
            .unwrap();
            let checking_account_nft_id = [7, 7, 7, 7, 7];
            let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();
            let script = pull_validator().unwrap();
            let input_tx_id = [8, 8, 8, 8];
            let account_input_tx_id = [9, 8, 7, 6];
            let input_tx_index = 0;
            let account_input_tx_index = 0;
            let script_address = script.address(NETWORK).unwrap();
            let spending_token = vec![5, 5, 5, 5];
            let input_datum = CheckingAccountDatums::AllowedPuller {
                next_pull: 10,
                period: 10,
                spending_token: spending_token.clone(),
                checking_account_address: checking_account_address.clone(),
                checking_account_nft: checking_account_nft_id.to_vec(),
            };
            let policy_id = hex::encode(&spending_token);
            let nft_id = hex::encode(&checking_account_nft_id);
            let output_datum = CheckingAccountDatums::AllowedPuller {
                next_pull: 20,
                period: 10,
                spending_token,
                checking_account_address: checking_account_address.clone(),
                checking_account_nft: checking_account_nft_id.to_vec(),
            };
            TestContext {
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
                account_output_address: checking_account_address,
                account_output_nft_id: nft_id.clone(),
                account_output_token_amt: 1,
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
            output_builder
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
                .finish_input()
                .with_output(&self.account_output_address)
                .with_value(
                    &self.account_output_nft_id,
                    "nft",
                    self.account_output_token_amt,
                )
                .finish_output()
                .build_spend(&self.input_tx_id, self.input_index)
        }
    }

    #[test]
    fn execute__happy_path() {
        let ctx_builder = TestContext::happy_path();
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let script = pull_validator().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap();
    }

    #[test]
    fn execute__before_next_pull_date_fails() {
        // given
        let mut ctx_builder = TestContext::happy_path();
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
        let mut ctx_builder = TestContext::happy_path();
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
        let mut ctx_builder = TestContext::happy_path();
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
        let mut ctx_builder = TestContext::happy_path();
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
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft,
            } => CheckingAccountDatums::AllowedPuller {
                next_pull: next_pull - 1,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft,
            },
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
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft,
            } => CheckingAccountDatums::AllowedPuller {
                next_pull,
                period: period + 1,
                spending_token,
                checking_account_address,
                checking_account_nft,
            },
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
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();
        let bad_spending_token = vec![6, 6, 6, 6];

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token: _,
                checking_account_address,
                checking_account_nft,
            } => CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token: bad_spending_token,
                checking_account_address,
                checking_account_nft,
            },
            _ => panic!("wrong variant"),
        };
        ctx_builder.output_datum = Some(new_datum);

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();

        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }

    #[test]
    fn execute__new_pull_datum_fails_if_account_address_changes() {
        // given
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();

        // when
        let bad_address =
            Address::from_bech32("addr_test1vzpwq95z3xyum8vqndgdd9mdnmafh3djcxnc6jemlgdmswcve6tkw")
                .unwrap();
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address: _,
                checking_account_nft,
            } => CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address: bad_address,
                checking_account_nft,
            },
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
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();
        let bad_nft_id = vec![5, 4, 3, 4, 1, 6, 2];

        // when
        let new_datum = match ctx_builder.output_datum.unwrap() {
            CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft: _,
            } => CheckingAccountDatums::AllowedPuller {
                next_pull,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft: bad_nft_id,
            },
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
        let mut ctx_builder = TestContext::happy_path();
        let script = pull_validator().unwrap();

        // when
        ctx_builder.output_token_policy_id = "".to_string(); // Replace spending token with lovelace

        //then
        let input_datum = ctx_builder.input_datum.clone().unwrap();
        let ctx = ctx_builder.build();
        let _eval = script.execute(input_datum, (), ctx).unwrap_err();
    }
}
