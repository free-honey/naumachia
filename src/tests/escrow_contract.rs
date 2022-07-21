use crate::address::ADA;
use crate::fakes::FakeBackendsBuilder;
use crate::smart_contract::{DataSource, SmartContract};
use crate::validator::{TxContext, ValidatorCode};
use crate::{Address, UnBuiltTransaction};
use std::collections::HashMap;

pub struct EscrowValidatorScript;

impl ValidatorCode for EscrowValidatorScript {
    fn execute<D, R>(_datum: D, _redeemer: R, _ctx: TxContext) -> bool {
        todo!()
    }

    fn address() -> Address {
        Address::new("escrow validator")
    }
}

struct EscrowContract;

enum Endpoint {
    Escrow { amount: u64, receiver: Address },
}

#[derive(Clone, PartialEq, Debug)]
struct EscrowDatum {
    receiver: Address,
}

impl SmartContract for EscrowContract {
    type Endpoint = Endpoint;
    type Datum = EscrowDatum;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        _source: &D,
    ) -> crate::Result<UnBuiltTransaction<EscrowDatum>> {
        match endpoint {
            Endpoint::Escrow { amount, receiver } => escrow(amount, receiver),
        }
    }
}

fn escrow(amount: u64, receiver: Address) -> crate::Result<UnBuiltTransaction<EscrowDatum>> {
    let address = EscrowValidatorScript::address();
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
    let backend = FakeBackendsBuilder::new(me.clone())
        .start_output(me)
        .with_value(ADA, 100)
        .finish_output()
        .build();

    let amount = 50;
    let call = Endpoint::Escrow {
        amount,
        receiver: alice,
    };
    EscrowContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();

    let address = EscrowValidatorScript::address();
    let expected = amount;
    let actual = backend.balance_at_address(&address, &ADA);
    assert_eq!(expected, actual)
}
