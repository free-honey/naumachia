use async_trait::async_trait;
use naumachia::{
    ledger_client::{
        test_ledger_client::{
            in_memory_storage::InMemoryStorage,
            TestLedgerClient,
            TestLedgerClientBuilder,
        },
        LedgerClient,
    },
    logic::{
        error::{
            SCLogicError,
            SCLogicResult,
        },
        SCLogic,
    },
    policy_id::PolicyId,
    scripts::{
        context::TxContext,
        ExecutionCost,
        MintingPolicy,
        ScriptResult,
    },
    smart_contract::{
        SmartContract,
        SmartContractTrait,
    },
    transaction::TxActions,
};
use pallas_addresses::Address;

struct AlwaysMintsPolicy;

impl<R> MintingPolicy<R> for AlwaysMintsPolicy {
    fn execute(&self, _redeemer: R, _ctx: TxContext) -> ScriptResult<ExecutionCost> {
        Ok(ExecutionCost::default())
    }

    fn id(&self) -> ScriptResult<String> {
        Ok(hex::encode(MINT_POLICY_ID))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AlwaysMintsSmartContract;

#[derive(Debug)]
enum Endpoint {
    Mint { amount: u64 },
}

const MINT_POLICY_ID: &[u8] = &[6, 6, 6, 6, 6];

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
                    .signer_base_address()
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

fn mint(amount: u64, _recipient: Address) -> SCLogicResult<TxActions<(), ()>> {
    let policy = Box::new(AlwaysMintsPolicy);
    let utx = TxActions::v1().with_mint(amount, None, (), policy);
    Ok(utx)
}

#[tokio::test]
async fn can_mint_from_always_true_minting_policy() {
    let me = Address::from_bech32("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm").unwrap();
    let policy = PolicyId::native_token(&hex::encode(MINT_POLICY_ID), &None);
    let backend = TestLedgerClientBuilder::new(&me).build_in_memory();
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    let contract = SmartContract::new(AlwaysMintsSmartContract, backend);
    contract.hit_endpoint(call).await.unwrap();

    // Check my balance for minted tokens
    let expected = amount;
    let actual = <TestLedgerClient<(), (), InMemoryStorage<()>> as LedgerClient<
        (),
        (),
    >>::balance_at_address(contract.ledger_client(), &me, &policy)
    .await
    .unwrap();
    assert_eq!(expected, actual)
}
