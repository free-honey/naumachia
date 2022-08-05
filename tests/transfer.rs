use naumachia::{
    address::{Address, ADA},
    backend::{fake_backend::FakeRecord, Backend, TxORecord},
    error::Result,
    output::Output,
    smart_contract::SmartContract,
    transaction::UnBuiltTransaction,
};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData};

struct TransferADASmartContract;

enum Endpoint {
    Transfer { amount: u64, recipient: Address },
}

impl SmartContract for TransferADASmartContract {
    type Endpoint = Endpoint;
    type Datum = ();
    type Redeemer = ();

    fn handle_endpoint(
        endpoint: Self::Endpoint,
        _issuer: &Address,
    ) -> Result<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Transfer { amount, recipient } => {
                let u_tx = UnBuiltTransaction::default().with_transfer(amount, recipient, ADA);
                Ok(u_tx)
            }
        }
    }
}

#[test]
fn can_transfer_and_keep_remainder() {
    let me = Address::new("me");
    let alice = Address::new("alice");

    let input_amount = 666;
    let mut values = HashMap::new();
    values.insert(ADA, input_amount);
    let extra_policy = Some(Address::new("arcade token"));
    let extra_amount = 50;
    values.insert(extra_policy.clone(), extra_amount);
    let input = Output::wallet(me.clone(), values.clone());

    let amount = 590;

    let txo_record = FakeRecord {
        signer: me.clone(),
        outputs: RefCell::new(vec![(me.clone(), input.clone())]),
    };
    let backend = Backend {
        smart_contract: TransferADASmartContract, 
        _datum: PhantomData::default(),
        _redeemer: PhantomData::default(),
        txo_record,
    };

    let call = Endpoint::Transfer {
        amount,
        recipient: alice.clone(),
    };

    backend.hit_endpoint( call).unwrap();

    let alice_expected = amount;
    let alice_actual = <FakeRecord<()> as TxORecord<(), ()>>::balance_at_address(
        &backend.txo_record,
        &alice,
        &ADA,
    );
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual =
        <FakeRecord<()> as TxORecord<(), ()>>::balance_at_address(&backend.txo_record, &me, &ADA);
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = <FakeRecord<()> as TxORecord<(), ()>>::balance_at_address(
        &backend.txo_record,
        &me,
        &extra_policy,
    );
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
