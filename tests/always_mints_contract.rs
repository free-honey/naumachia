use naumachia::address::Policy;
use naumachia::{
    address::Address,
    backend::{fake_backend::FakeRecord, Backend, TxORecord},
    error::Result,
    smart_contract::SmartContract,
    transaction::UnBuiltTransaction,
};
use std::cell::RefCell;
use std::marker::PhantomData;

struct AlwaysMintsSmartContract;

enum Endpoint {
    Mint {
        amount: u64, // TODO: Too big?
    },
}

const MINT_POLICY_ADDR: &str = "mint_policy";

impl SmartContract for AlwaysMintsSmartContract {
    type Endpoint = Endpoint;
    type Datum = ();
    type Redeemer = ();

    fn handle_endpoint(
        endpoint: Self::Endpoint,
        issuer: &Address,
    ) -> Result<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Mint { amount } => {
                let recipient = issuer.clone();
                mint(amount, recipient, Some(Address::new(MINT_POLICY_ADDR)))
            }
        }
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
    let txo_record: FakeRecord<()> = FakeRecord {
        signer: me.clone(),
        outputs: RefCell::new(vec![]),
    };
    let backend: Backend<(), (), FakeRecord<()>> = Backend {
        _datum: PhantomData::default(),
        _redeemer: PhantomData::default(),
        txo_record,
    };
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    Backend::hit_endpoint::<AlwaysMintsSmartContract>(&backend, call).unwrap();

    // Check my balance for minted tokens
    let expected = amount;
    let actual = <FakeRecord<()> as TxORecord<(), ()>>::balance_at_address(
        &backend.txo_record,
        &me,
        &policy,
    );
    assert_eq!(expected, actual)
}
