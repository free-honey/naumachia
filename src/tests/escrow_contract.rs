use crate::address::ADA;
use crate::fakes::{FakeBackends, FakeBackendsBuilder};
use crate::smart_contract::{DataSource, SmartContract};
use crate::validator::{TxContext, ValidatorCode};
use crate::{Address, UnBuiltTransaction};
use std::cell::RefCell;

pub struct EscrowValidatorScript;

impl ValidatorCode for EscrowValidatorScript {
    fn execute<D, R>(datum: D, redeemer: R, ctx: TxContext) -> bool {
        todo!()
    }

    fn address() -> Address {
        todo!()
    }
}

struct EscrowContract;

enum Endpoint {
    Escrow { amount: u64, receiver: Address },
}

impl SmartContract for EscrowContract {
    type Endpoint = Endpoint;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> crate::Result<UnBuiltTransaction> {
        match endpoint {
            Endpoint::Escrow { amount, receiver } => escrow(amount, receiver),
        }
    }
}

fn escrow(amount: u64, receiver: Address) -> crate::Result<UnBuiltTransaction> {
    todo!()
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

    // Call mint endpoint
    let amount = 50;
    let call = Endpoint::Escrow {
        amount,
        receiver: alice,
    };
    EscrowContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();
    // Wait 1 block? IDK if we need to wait. That's an implementation detail of a specific data
    // source I think? Could be wrong.

    // Check my balance for minted tokens
    let address = EscrowValidatorScript::address();
    let expected = amount;
    let actual = backend.balance_at_address(&address, &ADA);
    assert_eq!(expected, actual)
}
