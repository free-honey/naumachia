use async_trait::async_trait;
use naumachia::address::PolicyId;
use naumachia::ledger_client::in_memory_ledger::TestBackendsBuilder;
use naumachia::logic::SCLogicResult;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};
use naumachia::{
    address::Address, ledger_client::LedgerClient, logic::SCLogic, transaction::UnBuiltTransaction,
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct TransferADASmartContract;

enum Endpoint {
    Transfer { amount: u64, recipient: Address },
}

#[async_trait]
impl SCLogic for TransferADASmartContract {
    type Endpoint = Endpoint;
    type Lookup = ();
    type LookupResponse = ();
    type Datum = ();
    type Redeemer = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        _txo_record: &Record,
    ) -> SCLogicResult<UnBuiltTransaction<(), ()>> {
        match endpoint {
            Endpoint::Transfer { amount, recipient } => {
                let u_tx =
                    UnBuiltTransaction::default().with_transfer(amount, recipient, PolicyId::ADA);
                Ok(u_tx)
            }
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        _txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        Ok(())
    }
}

#[tokio::test]
async fn can_transfer_and_keep_remainder() {
    let me = Address::new("me");
    let alice = Address::new("alice");

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

    contract.hit_endpoint(call).await.unwrap();

    let alice_expected = amount;
    let alice_actual = backend
        .txo_record
        .balance_at_address(&alice, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual = backend
        .txo_record
        .balance_at_address(&me, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = backend
        .txo_record
        .balance_at_address(&me, &extra_policy)
        .await
        .unwrap();
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
