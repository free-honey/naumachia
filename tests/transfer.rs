use naumachia::address::{PolicyId, ValidAddress};
use naumachia::ledger_client::fake_address::FakeAddress;
use naumachia::ledger_client::in_memory_ledger::TestBackendsBuilder;
use naumachia::logic::SCLogicResult;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};
use naumachia::{ledger_client::LedgerClient, logic::SCLogic, transaction::UnBuiltTransaction};

#[derive(Debug, Clone, Eq, PartialEq)]
struct TransferADASmartContract;

enum Endpoint<Address> {
    Transfer { amount: u64, recipient: Address },
}

impl<Address: ValidAddress> SCLogic<Address> for TransferADASmartContract {
    type Endpoint = Endpoint<Address>;
    type Lookup = ();
    type LookupResponse = ();
    type Datum = ();
    type Redeemer = ();

    fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        _txo_record: &Record,
    ) -> SCLogicResult<UnBuiltTransaction<Address, (), ()>> {
        match endpoint {
            Endpoint::Transfer { amount, recipient } => {
                let u_tx =
                    UnBuiltTransaction::default().with_transfer(amount, &recipient, PolicyId::ADA);
                Ok(u_tx)
            }
        }
    }

    fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        _txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        Ok(())
    }
}

#[test]
fn can_transfer_and_keep_remainder() {
    let me = FakeAddress::new("me");
    let alice = FakeAddress::new("alice");

    let input_amount = 666;
    let extra_policy = PolicyId::native_token("arcade token");
    let extra_amount = 50;

    let amount = 590;

    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, input_amount)
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
    let alice_actual = backend
        .ledger_client
        .balance_at_address(&alice, &PolicyId::ADA);
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual = backend
        .ledger_client
        .balance_at_address(&me, &PolicyId::ADA);
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = backend.ledger_client.balance_at_address(&me, &extra_policy);
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
