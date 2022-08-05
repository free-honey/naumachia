use crate::{
    address::Address,
    backend::Backend,
    error,
    smart_contract::{DataSource, SmartContract},
    Policy, UnBuiltTransaction,
};
use std::cell::RefCell;
use std::marker::PhantomData;

use crate::backend::{FakeRecord, TxORecord};
use error::Result;

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

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> Result<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Mint { amount } => {
                let recipient = source.me().clone();
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
    let txo_record = FakeRecord {
        signer: me.clone(),
        outputs: RefCell::new(vec![]),
    };
    let backend = Backend {
        // signer: me.clone(),
        // outputs: RefCell::new(vec![]),
        _datum: PhantomData::default(),
        _redeemer: PhantomData::default(),
        txo_record,
    };
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    AlwaysMintsSmartContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();
    // Wait 1 block? IDK if we need to wait. That's an implementation detail of a specific data
    // source I think? Could be wrong.

    // Check my balance for minted tokens
    let expected = amount;
    let actual = backend
        .txo_record
        .balance_at_address(&me, &Some(Address::new(MINT_POLICY_ADDR))); // TODO: Use policy address
    assert_eq!(expected, actual)
}
