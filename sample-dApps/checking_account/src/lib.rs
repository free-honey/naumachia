use crate::scripts::checking_account_validtor::checking_account_validator;
use crate::scripts::spend_token_policy::spend_token_policy;
use crate::scripts::FakePullerValidator;
use async_trait::async_trait;
use nau_scripts::one_shot;
use nau_scripts::one_shot::OutputReference;
use naumachia::output::{Output, OutputId};
use naumachia::scripts::raw_validator_script::plutus_data::{Constr, PlutusData};
use naumachia::scripts::{MintingPolicy, ScriptError};
use naumachia::{
    address::{Address, PolicyId},
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    scripts::ValidatorCode,
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
        spending_token_id: String,
    },
    /// Remove funds from a checking account
    WithdrawFromAccount {
        output_id: OutputId,
        withdraw_amount: u64,
        spending_token_id: String,
    },
    /// Use allowed puller validator to pull from checking account
    PullFromCheckingAccount {
        allow_pull_output_id: OutputId,
        checking_account_output_id: OutputId,
        amount: u64,
        spending_token_id: String,
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
        puller: Address,
        amount_lovelace: u64,
        period: i64,
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
            CheckingAccountDatums::AllowedPuller { .. } => {
                todo!()
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
                puller,
                amount_lovelace,
                period,
                next_pull,
            } => add_puller(puller, amount_lovelace, period, next_pull).await,
            CheckingAccountEndpoints::RemovePuller { output_id } => {
                remove_puller(ledger_client, output_id).await
            }
            CheckingAccountEndpoints::FundAccount {
                output_id,
                fund_amount,
                spending_token_id,
            } => fund_account(ledger_client, output_id, fund_amount, spending_token_id).await,
            CheckingAccountEndpoints::WithdrawFromAccount {
                output_id,
                withdraw_amount,
                spending_token_id,
            } => {
                withdraw_from_account(ledger_client, output_id, withdraw_amount, spending_token_id)
                    .await
            }
            CheckingAccountEndpoints::PullFromCheckingAccount {
                allow_pull_output_id,
                checking_account_output_id,
                amount,
                spending_token_id,
            } => {
                pull_from_account(
                    ledger_client,
                    allow_pull_output_id,
                    checking_account_output_id,
                    amount,
                    spending_token_id,
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
    let validator_parameterized =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spending_token_id = spending_token_policy.id().unwrap();
    let spending_token_id_bytes = hex::decode(spending_token_id).unwrap();
    todo!();
    // let validator = validator_parameterized
    //     .apply(spending_token_id_bytes.into())
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    // let address = validator
    //     .address(0)
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    //
    // let datum = CheckingAccountDatums::CheckingAccount { owner };
    // let actions = TxActions::v2().with_script_init(datum, values, address);
    // Ok(actions)
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
    println!("input id: {:?}", selected.id());
    Ok(selected)
}

// TODO: Include minting puller token
async fn add_puller(
    puller: Address,
    amount_lovelace: u64,
    period: i64,
    next_pull: i64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, 0);
    let datum = CheckingAccountDatums::AllowedPuller {
        puller,
        amount_lovelace,
        period,
        next_pull,
    };
    let address = FakePullerValidator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2().with_script_init(datum, values, address);
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
    spending_token_id: String,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator_parameterized =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spending_token_id_bytes = hex::decode(spending_token_id).unwrap();
    todo!();
    // let validator = validator_parameterized
    //     .apply(spending_token_id_bytes.into())
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    // let address = validator
    //     .address(0)
    //     .map_err(SCLogicError::ValidatorScript)?;
    // let output = ledger_client
    //     .all_outputs_at_address(&address)
    //     .await
    //     .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
    //     .into_iter()
    //     .find(|o| o.id() == &output_id)
    //     .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    //
    // let new_datum = output
    //     .datum()
    //     .ok_or(CheckingAccountError::OutputNotFound(output_id))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .clone();
    // let redeemer = ();
    // let script = Box::new(validator);
    // let mut values = output.values().to_owned();
    // values.add_one_value(&PolicyId::ADA, amount);
    // let actions = TxActions::v2()
    //     .with_script_redeem(output, redeemer, script)
    //     .with_script_init(new_datum, values, address);
    // Ok(actions)
}

async fn withdraw_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
    amount: u64,
    spending_token_id: String,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator_parameterized =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spending_token_id_bytes = hex::decode(spending_token_id).unwrap();
    todo!();
    // let validator = validator_parameterized
    //     .apply(spending_token_id_bytes.into())
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    // let address = validator
    //     .address(0)
    //     .map_err(SCLogicError::ValidatorScript)?;
    // let output = ledger_client
    //     .all_outputs_at_address(&address)
    //     .await
    //     .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
    //     .into_iter()
    //     .find(|o| o.id() == &output_id)
    //     .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    //
    // let new_datum = output
    //     .datum()
    //     .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .clone();
    // let redeemer = ();
    // let script = Box::new(validator);
    // let old_values = output.values().to_owned();
    // let mut sub_values = Values::default();
    // sub_values.add_one_value(&PolicyId::ADA, amount);
    // let new_value = old_values
    //     .try_subtract(&sub_values)
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    // let actions = TxActions::v2()
    //     .with_script_redeem(output, redeemer, script)
    //     .with_script_init(new_datum, new_value, address);
    // Ok(actions)
}

async fn pull_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    allow_pull_output_id: OutputId,
    checking_account_output_id: OutputId,
    amount: u64,
    spending_token_id: String,
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

    let validator_parameterized =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spending_token_id_bytes = hex::decode(spending_token_id).unwrap();
    todo!();
    // let checking_account_validator = validator_parameterized
    //     .apply(spending_token_id_bytes.into())
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    // let checking_account_address = checking_account_validator
    //     .address(0)
    //     .map_err(SCLogicError::ValidatorScript)?;
    // let checking_account_output = ledger_client
    //     .all_outputs_at_address(&checking_account_address)
    //     .await
    //     .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
    //     .into_iter()
    //     .find(|o| o.id() == &checking_account_output_id)
    //     .ok_or(CheckingAccountError::OutputNotFound(
    //         checking_account_output_id.clone(),
    //     ))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    //
    // let new_allow_pull_datum = allow_pull_output
    //     .datum()
    //     .ok_or(CheckingAccountError::OutputNotFound(
    //         allow_pull_output_id.clone(),
    //     ))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .clone();
    // let allow_pull_redeemer = ();
    // let allow_pull_script = Box::new(allow_pull_validator);
    // let allow_pull_value = Values::default();
    //
    // let new_checking_account_datum = checking_account_output
    //     .datum()
    //     .ok_or(CheckingAccountError::OutputNotFound(
    //         checking_account_output_id.clone(),
    //     ))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .clone();
    // let checking_account_redeemer = ();
    // let checking_account_script = Box::new(checking_account_validator);
    //
    // let old_values = checking_account_output.values().to_owned();
    // let mut sub_values = Values::default();
    // sub_values.add_one_value(&PolicyId::ADA, amount);
    // let new_account_value = old_values
    //     .try_subtract(&sub_values)
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
    //     .ok_or(CheckingAccountError::OutputNotFound(
    //         checking_account_output_id.clone(),
    //     ))
    //     .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    //
    // let actions = TxActions::v2()
    //     .with_script_redeem(allow_pull_output, allow_pull_redeemer, allow_pull_script)
    //     .with_script_init(new_allow_pull_datum, allow_pull_value, allow_pull_address)
    //     .with_script_redeem(
    //         checking_account_output,
    //         checking_account_redeemer,
    //         checking_account_script,
    //     )
    //     .with_script_init(
    //         new_checking_account_datum,
    //         new_account_value,
    //         checking_account_address,
    //     );
    // Ok(actions)
}
