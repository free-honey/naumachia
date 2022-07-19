use crate::fakes::FakeBackends;
use crate::{Address, DataSource, Output, SmartContract, UnBuiltTransaction, ADA};
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
        _source: &D,
    ) -> crate::Result<UnBuiltTransaction> {
        match endpoint {
            Endpoint::Transfer { amount, recipient } => {
                let u_tx = UnBuiltTransaction::new().with_transfer(amount, recipient, ADA);
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
    let alice_actual = backend.balance_at_address(&alice, &ADA);
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual = backend.balance_at_address(&me, &ADA);
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = backend.balance_at_address(&me, &extra_policy);
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
