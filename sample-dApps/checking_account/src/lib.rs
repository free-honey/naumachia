use crate::scripts::{
    checking_account_validtor::checking_account_validator, spend_token_policy::spend_token_policy,
    FakePullerValidator,
};
use async_trait::async_trait;
use nau_scripts::{one_shot, one_shot::OutputReference};
use naumachia::{
    address::{Address, PolicyId},
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    output::{Output, OutputId},
    scripts::{
        raw_validator_script::plutus_data::{Constr, PlutusData},
        MintingPolicy, ScriptError, ValidatorCode,
    },
    transaction::TxActions,
    values::Values,
};
use thiserror::Error;

pub mod scripts;

#[allow(non_snake_case)]
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum CheckingAccountEndpoints {
    // Owner Endpoints
    /// Create a new checking account
    InitAccount { starting_lovelace: u64 },
    /// Allow puller to pull amount from checking account every period,
    /// starting on the next_pull time, in milliseconds POSIX
    AddPuller {
        checking_account_nft: String,
        puller: Address,
        amount_lovelace: u64,
        period: i64,
        next_pull: i64,
    },
    /// Disallow puller from accessing account account
    RemovePuller { output_id: OutputId },
    /// Add funds to checking account
    FundAccount {
        output_id: OutputId,
        fund_amount: u64,
    },
    /// Remove funds from a checking account
    WithdrawFromAccount {
        output_id: OutputId,
        withdraw_amount: u64,
    },
    /// Use allowed puller validator to pull from checking account
    PullFromCheckingAccount {
        allow_pull_output_id: OutputId,
        checking_account_output_id: OutputId,
        amount: u64,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CheckingAccountLogic;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CheckingAccountDatums {
    CheckingAccount {
        owner: Address,
        spend_token_policy: String,
    },
    AllowedPuller {
        // puller: Address,
        // amount_lovelace: u64,
        // period: i64,
        next_pull: i64,
    },
}

impl From<CheckingAccountDatums> for PlutusData {
    fn from(value: CheckingAccountDatums) -> Self {
        match value {
            CheckingAccountDatums::CheckingAccount {
                owner,
                spend_token_policy,
            } => {
                let owner_data = PlutusData::BoundedBytes(owner.bytes().unwrap()); // TODO
                let policy_data =
                    PlutusData::BoundedBytes(hex::decode(spend_token_policy).unwrap()); // TODO
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: vec![owner_data, policy_data],
                })
            }
            CheckingAccountDatums::AllowedPuller { next_pull } => {
                let next_pull = PlutusData::BigInt(next_pull.into());
                PlutusData::Constr(Constr {
                    tag: 121,
                    any_constructor: None,
                    fields: vec![next_pull],
                })
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum CheckingAccountError {
    #[error("Could not find a valid input UTxO")]
    InputNotFound,
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
    #[error("Expected datum on output with id: {0:?}")]
    DatumNotFoundForOutput(OutputId),
    #[error("You are trying to withdraw more than is in account")]
    CannotWithdrawSpecifiedAmount,
}

#[async_trait]
impl SCLogic for CheckingAccountLogic {
    type Endpoints = CheckingAccountEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = CheckingAccountDatums;
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            CheckingAccountEndpoints::InitAccount { starting_lovelace } => {
                init_account(ledger_client, starting_lovelace).await
            }
            CheckingAccountEndpoints::AddPuller {
                checking_account_nft,
                puller,
                amount_lovelace,
                period,
                next_pull,
            } => {
                add_puller(
                    ledger_client,
                    checking_account_nft,
                    puller,
                    amount_lovelace,
                    period,
                    next_pull,
                )
                .await
            }
            CheckingAccountEndpoints::RemovePuller { output_id } => {
                remove_puller(ledger_client, output_id).await
            }
            CheckingAccountEndpoints::FundAccount {
                output_id,
                fund_amount,
            } => fund_account(ledger_client, output_id, fund_amount).await,
            CheckingAccountEndpoints::WithdrawFromAccount {
                output_id,
                withdraw_amount,
            } => withdraw_from_account(ledger_client, output_id, withdraw_amount).await,
            CheckingAccountEndpoints::PullFromCheckingAccount {
                allow_pull_output_id,
                checking_account_output_id,
                amount,
            } => {
                pull_from_account(
                    ledger_client,
                    allow_pull_output_id,
                    checking_account_output_id,
                    amount,
                )
                .await
            }
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        _query: Self::Lookups,
        _ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        todo!()
    }
}

async fn init_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    starting_lovelace: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let owner = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, starting_lovelace);
    let my_input = select_any_above_min(ledger_client).await?;
    let nft = one_shot::get_parameterized_script().map_err(SCLogicError::PolicyScript)?;
    let nft_policy = nft
        .apply(OutputReference::from(&my_input))
        .map_err(|e| ScriptError::FailedToConstruct(format!("{:?}", e)))
        .map_err(SCLogicError::PolicyScript)?;
    let spending_token_policy_parameterized =
        spend_token_policy().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let script_id = nft_policy.id().unwrap();
    let script_id_bytes = hex::decode(script_id).unwrap();
    let spending_token_policy = spending_token_policy_parameterized
        .apply(script_id_bytes.into())
        .unwrap()
        .apply(owner.clone().into())
        .unwrap();
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spend_token_policy = spending_token_policy.id().unwrap();
    let address = validator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let datum = CheckingAccountDatums::CheckingAccount {
        owner,
        spend_token_policy,
    };
    let actions = TxActions::v2().with_script_init(datum, values, address);
    Ok(actions)
}

async fn select_any_above_min<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
) -> SCLogicResult<Output<CheckingAccountDatums>> {
    const MIN_LOVELACE: u64 = 5_000_000;
    let me = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let selected = ledger_client
        .all_outputs_at_address(&me)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .iter()
        .filter_map(|input| {
            if let Some(ada_value) = input.values().get(&PolicyId::ADA) {
                if ada_value > MIN_LOVELACE {
                    Some(input)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .pop()
        .ok_or(SCLogicError::Endpoint(Box::new(
            CheckingAccountError::InputNotFound,
        )))?
        .to_owned();
    Ok(selected)
}

pub const SPEND_TOKEN_ASSET_NAME: &str = "SPEND TOKEN";

async fn add_puller<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    checking_account_nft_id: String,
    _puller: Address,
    _amount_lovelace: u64,
    _period: i64,
    next_pull: i64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let me = ledger_client
        .signer()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let datum = CheckingAccountDatums::AllowedPuller {
        // puller,
        // amount_lovelace,
        // period,
        next_pull,
    };
    let parameterized_spending_token_policy = spend_token_policy().unwrap();
    let nft_id_bytes = hex::decode(checking_account_nft_id).unwrap();
    let policy = parameterized_spending_token_policy
        .apply(nft_id_bytes.into())
        .unwrap()
        .apply(me.into())
        .unwrap();
    let id = policy.id().unwrap();
    let boxed_policy = Box::new(policy);

    let address = FakePullerValidator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let mut values = Values::default();
    values.add_one_value(
        &PolicyId::NativeToken(id, Some(SPEND_TOKEN_ASSET_NAME.to_string())),
        1,
    );
    let actions = TxActions::v2()
        .with_mint(
            1,
            Some(SPEND_TOKEN_ASSET_NAME.to_string()),
            (),
            boxed_policy,
        )
        .with_script_init(datum, values, address);
    Ok(actions)
}

async fn remove_puller<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator = FakePullerValidator;
    let address = validator
        .address(0)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script = Box::new(validator);
    let actions = TxActions::v2().with_script_redeem(output, redeemer, script);
    Ok(actions)
}

async fn fund_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
    amount: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let address = validator
        .address(0)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let new_datum = output
        .datum()
        .ok_or(CheckingAccountError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let redeemer = ();
    let script = Box::new(validator);
    let mut values = output.values().to_owned();
    values.add_one_value(&PolicyId::ADA, amount);
    let actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script)
        .with_script_init(new_datum, values, address);
    Ok(actions)
}

async fn withdraw_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
    amount: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let address = validator
        .address(0)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let new_datum = output
        .datum()
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let redeemer = ();
    let script = Box::new(validator);
    let old_values = output.values().to_owned();
    let mut sub_values = Values::default();
    sub_values.add_one_value(&PolicyId::ADA, amount);
    let new_value = old_values
        .try_subtract(&sub_values)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script)
        .with_script_init(new_datum, new_value, address);
    Ok(actions)
}

async fn pull_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    allow_pull_output_id: OutputId,
    checking_account_output_id: OutputId,
    amount: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let allow_pull_validator = FakePullerValidator;
    let allow_pull_address = allow_pull_validator
        .address(0)
        .map_err(SCLogicError::ValidatorScript)?;
    let allow_pull_output = ledger_client
        .all_outputs_at_address(&allow_pull_address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &allow_pull_output_id)
        .ok_or(CheckingAccountError::OutputNotFound(
            allow_pull_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let checking_account_address = validator
        .address(0)
        .map_err(SCLogicError::ValidatorScript)?;
    let checking_account_output = ledger_client
        .all_outputs_at_address(&checking_account_address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &checking_account_output_id)
        .ok_or(CheckingAccountError::OutputNotFound(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let new_allow_pull_datum = allow_pull_output
        .datum()
        .ok_or(CheckingAccountError::OutputNotFound(
            allow_pull_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let allow_pull_redeemer = ();
    let allow_pull_script = Box::new(allow_pull_validator);
    let allow_pull_value = Values::default();

    let new_checking_account_datum = checking_account_output
        .datum()
        .ok_or(CheckingAccountError::OutputNotFound(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let checking_account_redeemer = ();
    let checking_account_script = Box::new(validator);

    let old_values = checking_account_output.values().to_owned();
    let mut sub_values = Values::default();
    sub_values.add_one_value(&PolicyId::ADA, amount);
    let new_account_value = old_values
        .try_subtract(&sub_values)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .ok_or(CheckingAccountError::OutputNotFound(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let actions = TxActions::v2()
        .with_script_redeem(allow_pull_output, allow_pull_redeemer, allow_pull_script)
        .with_script_init(new_allow_pull_datum, allow_pull_value, allow_pull_address)
        .with_script_redeem(
            checking_account_output,
            checking_account_redeemer,
            checking_account_script,
        )
        .with_script_init(
            new_checking_account_datum,
            new_account_value,
            checking_account_address,
        );
    Ok(actions)
}
