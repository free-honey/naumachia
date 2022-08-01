use crate::output::Output;
use crate::{
    address::ADA,
    fakes::FakeBackendsBuilder,
    smart_contract::{DataSource, SmartContract},
    validator::{TxContext, ValidatorCode},
    Address, UnBuiltTransaction,
};
use std::collections::HashMap;

pub struct EscrowValidatorScript;

impl<D, R> ValidatorCode<D, R> for EscrowValidatorScript {
    fn execute(&self, _datum: D, _redeemer: R, _ctx: TxContext) -> bool {
        todo!()
    }

    fn address(&self) -> Address {
        Address::new("escrow validator")
    }
}

struct EscrowContract;

#[derive(Clone)]
enum Endpoint {
    Escrow { amount: u64, receiver: Address },
    Claim { output: Output<EscrowDatum> },
}

#[derive(Clone, PartialEq, Debug)]
struct EscrowDatum {
    receiver: Address,
}

impl SmartContract for EscrowContract {
    type Endpoint = Endpoint;
    type Datum = EscrowDatum;
    type Redeemer = ();

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        _source: &D,
    ) -> crate::Result<UnBuiltTransaction<EscrowDatum, ()>> {
        match endpoint {
            Endpoint::Escrow { amount, receiver } => escrow(amount, receiver),
            Endpoint::Claim { .. } => Err("bonk".to_string()),
        }
    }
}

fn escrow(amount: u64, receiver: Address) -> crate::Result<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = EscrowValidatorScript;
    let address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let datum = EscrowDatum { receiver };
    let mut values = HashMap::new();
    values.insert(ADA, amount);
    let u_tx = UnBuiltTransaction::default().with_script_init(datum, values, address);
    Ok(u_tx)
}

#[test]
fn escrow__can_create_instance() {
    let me = Address::new("me");
    let alice = Address::new("alice");
    let mut backend = FakeBackendsBuilder::new(me.clone())
        .start_output(me)
        .with_value(ADA, 100)
        .finish_output()
        .build();

    let escrow_amount = 50;
    let call = Endpoint::Escrow {
        amount: escrow_amount,
        receiver: alice.clone(),
    };
    let script = EscrowValidatorScript;
    EscrowContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();

    let escrow_address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let expected = escrow_amount;
    let actual = backend.balance_at_address(&escrow_address, &ADA);
    assert_eq!(expected, actual);

    let instance = backend.outputs_at_address(&escrow_address).pop().unwrap();
    dbg!(&instance);
    // The creator tries to spend escrow but fails because not recipient
    let call = Endpoint::Claim { output: instance };

    let attempt = EscrowContract::hit_endpoint(call.clone(), &backend, &backend, &backend);
    assert!(attempt.is_err());

    // The recipient tries to spend and succeeds
    backend.signer = alice.clone();
    let attempt = EscrowContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();

    let alice_balance = backend.balance_at_address(&alice, &ADA);
    assert_eq!(alice_balance, escrow_amount);
}
