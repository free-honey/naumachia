use crate::scripts::{FakeCheckingAccountValidator, FakePullerValidator};
use async_trait::async_trait;
use naumachia::{
    address::{Address, PolicyId},
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};

pub mod scripts;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum CheckingAccountEndpoints {
    // Owner Endpoints
    /// Create a new checking account
    InitAccount {
        starting_lovelace: u64,
    },
    /// Allow puller to pull amount from checking account every period,
    /// starting on the next_pull time, in milliseconds POSIX
    AddPuller {
        puller: Address,
        amount_lovelace: u64,
        period: i64,
        next_pull: i64,
    },
    RemovePuller,
    FundAccount,
    WithdrawFromAccount,
    // PullerEndpoints
    PullFromCheckingAccount,
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
            } => add_puller(puller, amount_lovelace, period, next_pull),
            _ => todo!(),
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
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
    let datum = CheckingAccountDatums::CheckingAccount { owner };
    let address = FakeCheckingAccountValidator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2().with_script_init(datum, values, address);
    Ok(actions)
}

fn add_puller(
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
}
