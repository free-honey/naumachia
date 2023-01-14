use crate::scripts::{FakeCheckingAccountValidator, FakePullerValidator};
use async_trait::async_trait;
use naumachia::output::OutputId;
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
    },
    AllowedPuller {
        puller: Address,
        amount_lovelace: u64,
        period: i64,
        next_pull: i64,
    },
}

#[derive(Debug, Error)]
pub enum CheckingAccountError {
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

// TODO: Include account NFT?
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
    let datum = CheckingAccountDatums::CheckingAccount { owner };
    let address = FakeCheckingAccountValidator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2().with_script_init(datum, values, address);
    Ok(actions)
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
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let validator = FakeCheckingAccountValidator;
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
    let validator = FakeCheckingAccountValidator;
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

    let checking_account_validator = FakeCheckingAccountValidator;
    let checking_account_address = checking_account_validator
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
    let checking_account_script = Box::new(checking_account_validator);

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

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::{FakeCheckingAccountValidator, FakePullerValidator};
    use naumachia::address::{Address, PolicyId};
    use naumachia::ledger_client::test_ledger_client::TestBackendsBuilder;
    use naumachia::smart_contract::{SmartContract, SmartContractTrait};

    const NETWORK: u8 = 0;

    #[tokio::test]
    async fn init_creates_instance_with_correct_balance() {
        let me = Address::new("me");
        let start_amount = 100_000_000;
        let backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build_in_memory();

        let account_amount = 10_000_000;
        let endpoint = CheckingAccountEndpoints::InitAccount {
            starting_lovelace: account_amount,
        };
        let script = FakeCheckingAccountValidator;
        let contract = SmartContract::new(&CheckingAccountLogic, &backend);
        contract.hit_endpoint(endpoint).await.unwrap();

        let address = script.address(NETWORK).unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, account_amount);
    }

    #[tokio::test]
    async fn add_puller_creates_new_datum_for_puller() {
        let me = Address::new("me");
        let start_amount = 100_000_000;
        let backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build_in_memory();

        let puller = Address::new("puller");
        let endpoint = CheckingAccountEndpoints::AddPuller {
            puller: puller.clone(),
            amount_lovelace: 15_000_000,
            period: 1000,
            next_pull: 0,
        };
        let contract = SmartContract::new(&CheckingAccountLogic, &backend);
        contract.hit_endpoint(endpoint).await.unwrap();
        let script = FakePullerValidator;
        let address = script.address(NETWORK).unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, 0);
    }

    #[tokio::test]
    async fn remove_puller__removes_the_allowed_puller() {
        let me = Address::new("me");
        let start_amount = 100_000_000;
        let backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build_in_memory();

        let puller = Address::new("puller");
        let add_endpoint = CheckingAccountEndpoints::AddPuller {
            puller: puller.clone(),
            amount_lovelace: 15_000_000,
            period: 1000,
            next_pull: 0,
        };

        let contract = SmartContract::new(&CheckingAccountLogic, &backend);
        contract.hit_endpoint(add_endpoint).await.unwrap();
        let script = FakePullerValidator;
        let address = script.address(NETWORK).unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let output_id = script_output.id().to_owned();

        let remove_endpoint = CheckingAccountEndpoints::RemovePuller { output_id };

        contract.hit_endpoint(remove_endpoint).await.unwrap();

        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        assert!(outputs_at_address.pop().is_none());
    }

    #[tokio::test]
    async fn fund_account__replaces_existing_balance_with_updated_amount() {
        let me = Address::new("me");
        let start_amount = 100_000_000;
        let backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build_in_memory();

        let account_amount = 10_000_000;
        let fund_amount = 5_000_000;
        let init_endpoint = CheckingAccountEndpoints::InitAccount {
            starting_lovelace: account_amount,
        };
        let script = FakeCheckingAccountValidator;
        let contract = SmartContract::new(&CheckingAccountLogic, &backend);
        contract.hit_endpoint(init_endpoint).await.unwrap();

        let address = script.address(NETWORK).unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let output_id = script_output.id().to_owned();

        let fund_endpoint = CheckingAccountEndpoints::FundAccount {
            output_id,
            fund_amount,
        };

        contract.hit_endpoint(fund_endpoint).await.unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, account_amount + fund_amount);
    }

    #[tokio::test]
    async fn withdraw_from_account__replaces_existing_balance_with_updated_amount() {
        let me = Address::new("me");
        let start_amount = 100_000_000;
        let backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(PolicyId::ADA, start_amount)
            .finish_output()
            .build_in_memory();

        let account_amount = 10_000_000;
        let withdraw_amount = 5_000_000;
        let init_endpoint = CheckingAccountEndpoints::InitAccount {
            starting_lovelace: account_amount,
        };
        let script = FakeCheckingAccountValidator;
        let contract = SmartContract::new(&CheckingAccountLogic, &backend);
        contract.hit_endpoint(init_endpoint).await.unwrap();

        let address = script.address(NETWORK).unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let output_id = script_output.id().to_owned();

        let fund_endpoint = CheckingAccountEndpoints::WithdrawFromAccount {
            output_id,
            withdraw_amount,
        };

        contract.hit_endpoint(fund_endpoint).await.unwrap();
        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, account_amount - withdraw_amount);
    }

    #[tokio::test]
    async fn pull_from_account__replaces_existing_balances_with_updated_amounts() {
        let owner = Address::new("owner");
        let puller = Address::new("puller");

        let allow_puller_script = FakePullerValidator;
        let allow_puller_address = allow_puller_script.address(NETWORK).unwrap();
        let account_script = FakeCheckingAccountValidator;
        let account_address = account_script.address(NETWORK).unwrap();

        let account_amount = 100_000_000;
        let pull_amount = 15_000_000;
        let account_datum = CheckingAccountDatums::CheckingAccount { owner };
        let allow_puller_datum = CheckingAccountDatums::AllowedPuller {
            puller: puller.clone(),
            amount_lovelace: pull_amount,
            period: 1000,
            next_pull: 0,
        };
        let backend = TestBackendsBuilder::new(&puller)
            .start_output(&account_address)
            .with_datum(account_datum)
            .with_value(PolicyId::ADA, account_amount)
            .finish_output()
            .start_output(&allow_puller_address)
            .with_datum(allow_puller_datum)
            .with_value(PolicyId::ADA, 0)
            .finish_output()
            .build_in_memory();

        let contract = SmartContract::new(&CheckingAccountLogic, &backend);

        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&account_address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let checking_account_output_id = script_output.id().to_owned();

        let mut outputs_at_address = backend
            .ledger_client
            .all_outputs_at_address(&allow_puller_address)
            .await
            .unwrap();
        let script_output = outputs_at_address.pop().unwrap();
        let allow_pull_output_id = script_output.id().to_owned();

        // When
        let pull_endpoint = CheckingAccountEndpoints::PullFromCheckingAccount {
            allow_pull_output_id,
            checking_account_output_id,
            amount: pull_amount,
        };
        contract.hit_endpoint(pull_endpoint).await.unwrap();

        // Then
        let mut outputs_at_account_address = backend
            .ledger_client
            .all_outputs_at_address(&account_address)
            .await
            .unwrap();
        let script_output = outputs_at_account_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, account_amount - pull_amount);

        let mut outputs_at_puller_address = backend
            .ledger_client
            .all_outputs_at_address(&puller)
            .await
            .unwrap();
        let script_output = outputs_at_puller_address.pop().unwrap();
        let value = script_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(value, pull_amount);
    }
}
