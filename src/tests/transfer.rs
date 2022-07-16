use crate::tests::FakeBackends;
use crate::{Address, DataSource, Output, Policy, SmartContract, UnBuiltTransaction, ADA};
use std::cell::RefCell;
use std::collections::HashMap;

struct TransferADASmartContract;

enum Endpoint {
    Transfer { amount: u64, recipient: Address },
}

impl SmartContract for TransferADASmartContract {
    type Endpoint = Endpoint;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> crate::Result<UnBuiltTransaction> {
        let u_tx = UnBuiltTransaction::new();
        Ok(u_tx)
    }
}

#[test]
fn can_transfer_and_get_remainder() {
    let me = Address::new("me");
    let alice = Address::new("alice");

    let input_amount = 666;
    let mut values = HashMap::new();
    values.insert(ADA, input_amount);
    let input = Output {
        owner: me.clone(),
        values: values.clone(),
    };

    let amount = 590;

    let backend = FakeBackends {
        me: me.clone(),
        outputs: RefCell::new(vec![(me.clone(), input)]),
    };

    let call = Endpoint::Transfer {
        amount,
        recipient: alice.clone(),
    };

    TransferADASmartContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();

    let alice_expected = amount;
    let me_expected = input_amount - amount;

    let alice_actual = backend.balance_at_address(&alice, &ADA);
    let me_actual = backend.balance_at_address(&me, &ADA);
    assert_eq!(alice_expected, alice_actual);
    assert_eq!(me_expected, me_actual);
}
