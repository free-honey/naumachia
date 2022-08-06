use naumachia::backend::fake_backend::TestBackendsBuilder;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};
use naumachia::{
    address::{Address, ADA},
    backend::TxORecord,
    error::Result,
    logic::SCLogic,
    transaction::UnBuiltTransaction,
};

struct TransferADASmartContract;

enum Endpoint {
    Transfer { amount: u64, recipient: Address },
}

impl SCLogic for TransferADASmartContract {
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
    let extra_policy = Some(Address::new("arcade token"));
    let extra_amount = 50;

    let amount = 590;

    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(ADA, input_amount)
        .with_value(extra_policy.clone(), extra_amount)
        .finish_output()
        .build();

    let contract = SmartContract::new(&TransferADASmartContract, &backend);

    let call = Endpoint::Transfer {
        amount,
        recipient: alice.clone(),
    };

    contract.hit_endpoint(call).unwrap();

    let alice_expected = amount;
    let alice_actual = backend.txo_record.balance_at_address(&alice, &ADA);
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual = backend.txo_record.balance_at_address(&me, &ADA);
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = backend.txo_record.balance_at_address(&me, &extra_policy);
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
