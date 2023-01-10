use crate::scripts::FakeCheckingAccountValidator;
use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::ledger_client::LedgerClient;
use naumachia::logic::{SCLogic, SCLogicError, SCLogicResult};
use naumachia::scripts::ValidatorCode;
use naumachia::transaction::TxActions;
use naumachia::values::Values;

pub mod scripts;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

pub enum CheckingAccountEndpoints {
    // Owner Endpoints
    InitAccount { starting_lovelace: u64 },
    AddPuller,
    RemovePuller,
    FundAccount,
    WithdrawFromAccount,
    // PullerEndpoints
    PullFromCheckingAccount,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CheckingAccountLogic;

#[async_trait]
impl SCLogic for CheckingAccountLogic {
    type Endpoints = CheckingAccountEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            CheckingAccountEndpoints::InitAccount { starting_lovelace } => {
                init_account(starting_lovelace)
            }
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

fn init_account(starting_lovelace: u64) -> SCLogicResult<TxActions<(), ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, starting_lovelace);
    let address = FakeCheckingAccountValidator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2().with_script_init((), values, address);
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::FakeCheckingAccountValidator;
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
}
