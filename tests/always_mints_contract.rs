use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::ledger_client::test_ledger_client::{
    in_memory_storage::InMemoryStorage, TestBackendsBuilder, TestLedgerClient,
};
use naumachia::logic::SCLogicError;
use naumachia::{
    address::Address,
    ledger_client::LedgerClient,
    logic::SCLogic,
    logic::SCLogicResult,
    scripts::{MintingPolicy, ScriptResult, TxContext},
    smart_contract::SmartContract,
    smart_contract::SmartContractTrait,
    transaction::TxActions,
};

struct AlwaysMintsPolicy;

impl MintingPolicy for AlwaysMintsPolicy {
    fn execute(&self, _ctx: TxContext) -> ScriptResult<()> {
        Ok(())
    }

    fn id(&self) -> String {
        MINT_POLICY_ID.to_string()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AlwaysMintsSmartContract;

enum Endpoint {
    Mint { amount: u64 },
}

const MINT_POLICY_ID: &str = "mint_policy";

#[async_trait]
impl SCLogic for AlwaysMintsSmartContract {
    type Endpoints = Endpoint;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        txo_record: &Record,
    ) -> SCLogicResult<TxActions<(), ()>> {
        match endpoint {
            Endpoint::Mint { amount } => {
                let recipient = txo_record
                    .signer()
                    .await
                    .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
                mint(amount, recipient)
            }
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        _endpoint: Self::Lookups,
        _txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        Ok(())
    }
}

fn mint(amount: u64, recipient: Address) -> SCLogicResult<TxActions<(), ()>> {
    let policy = Box::new(AlwaysMintsPolicy);
    let utx = TxActions::default().with_mint(amount, None, &recipient, (), policy);
    Ok(utx)
}

#[tokio::test]
async fn can_mint_from_always_true_minting_policy() {
    let me = Address::new("me");
    let policy = PolicyId::native_token(MINT_POLICY_ID, &None);
    let backend = TestBackendsBuilder::new(&me).build_in_memory();
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    let contract = SmartContract::new(&AlwaysMintsSmartContract, &backend);
    contract.hit_endpoint(call).await.unwrap();

    // Check my balance for minted tokens
    let expected = amount;
    let actual = <TestLedgerClient<(), (), InMemoryStorage<()>> as LedgerClient<(), ()>>::balance_at_address(
        &backend.ledger_client,
        &me,
        &policy,
    )
    .await
    .unwrap();
    assert_eq!(expected, actual)
}
