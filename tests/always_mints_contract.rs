use naumachia::ledger_client::in_memory_ledger::{InMemoryLedgerClient, TestBackendsBuilder};
use naumachia::{
    address::Address,
    ledger_client::LedgerClient,
    logic::SCLogic,
    logic::SCLogicResult,
    scripts::{MintingPolicy, ScriptResult, TxContext},
    smart_contract::SmartContract,
    smart_contract::SmartContractTrait,
    transaction::UnBuiltTransaction,
};

struct AlwaysMintsPolicy;

impl MintingPolicy for AlwaysMintsPolicy {
    fn execute(&self, _ctx: TxContext) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self) -> Address {
        Address::new(MINT_POLICY_ADDR)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AlwaysMintsSmartContract;

enum Endpoint {
    Mint {
        amount: u64, // TODO: Too big?
    },
}

const MINT_POLICY_ADDR: &str = "mint_policy";

impl SCLogic for AlwaysMintsSmartContract {
    type Endpoint = Endpoint;
    type Lookup = ();
    type LookupResponse = ();
    type Datum = ();
    type Redeemer = ();

    fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> SCLogicResult<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Mint { amount } => {
                let recipient = txo_record.signer().clone();
                mint(amount, recipient)
            }
        }
    }

    fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        _txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        Ok(())
    }
}

fn mint(amount: u64, recipient: Address) -> SCLogicResult<UnBuiltTransaction<(), ()>> {
    let policy = Box::new(AlwaysMintsPolicy);
    let utx = UnBuiltTransaction::default().with_mint(amount, &recipient, policy);
    Ok(utx)
}

#[test]
fn can_mint_from_always_true_minting_policy() {
    let me = Address::new("me");
    let policy = Some(Address::new(MINT_POLICY_ADDR));
    let backend = TestBackendsBuilder::new(&me).build();
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    let contract = SmartContract::new(&AlwaysMintsSmartContract, &backend);
    contract.hit_endpoint(call).unwrap();

    // Check my balance for minted tokens
    let expected = amount;
    let actual = <InMemoryLedgerClient<(), ()> as LedgerClient<(), ()>>::balance_at_address(
        &backend.txo_record,
        &me,
        &policy,
    );
    assert_eq!(expected, actual)
}
