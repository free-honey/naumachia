use naumachia::backend::in_memory_record::TestBackendsBuilder;
use naumachia::smart_contract::SmartContractTrait;
use naumachia::{
    address::Address,
    address::Policy,
    backend::{in_memory_record::InMemoryRecord},
    txorecord::TxORecord,
    error::Result,
    logic::SCLogic,
    smart_contract::SmartContract,
    transaction::UnBuiltTransaction,
};

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

    fn handle_endpoint<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> Result<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Mint { amount } => {
                let recipient = txo_record.signer().clone();
                mint(amount, recipient, Some(Address::new(MINT_POLICY_ADDR)))
            }
        }
    }

    fn lookup<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        _txo_record: &Record,
    ) -> Result<Self::LookupResponse> {
        Ok(())
    }
}

fn mint(amount: u64, recipient: Address, policy: Policy) -> Result<UnBuiltTransaction<(), ()>> {
    let utx = UnBuiltTransaction::default().with_mint(amount, recipient, policy);
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
    let actual = <InMemoryRecord<(), ()> as TxORecord<(), ()>>::balance_at_address(
        &backend.txo_record,
        &me,
        &policy,
    );
    assert_eq!(expected, actual)
}
