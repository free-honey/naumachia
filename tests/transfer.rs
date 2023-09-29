use async_trait::async_trait;
use naumachia::{
    ledger_client::test_ledger_client::TestLedgerClientBuilder,
    ledger_client::LedgerClient,
    logic::SCLogic,
    logic::SCLogicResult,
    policy_id::PolicyId,
    smart_contract::{SmartContract, SmartContractTrait},
    transaction::TxActions,
};
use pallas_addresses::Address;

#[derive(Debug, Clone, Eq, PartialEq)]
struct TransferADASmartContract;

enum Endpoint {
    Transfer { amount: u64, recipient: Address },
}

#[async_trait]
impl SCLogic for TransferADASmartContract {
    type Endpoints = Endpoint;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        _txo_record: &Record,
    ) -> SCLogicResult<TxActions<(), ()>> {
        match endpoint {
            Endpoint::Transfer { amount, recipient } => {
                let u_tx = TxActions::v1().with_transfer(amount, recipient, PolicyId::Lovelace);
                Ok(u_tx)
            }
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        _endpoint: Self::Lookups,
        _txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        Ok(())
    }
}

#[tokio::test]
async fn can_transfer_and_keep_remainder() {
    let me = Address::from_bech32("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm").unwrap();
    let alice = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();

    let input_amount = 666;
    let extra_policy = PolicyId::native_token("arcade token", &None);
    let extra_amount = 50;

    let amount = 590;

    let ledger_client = TestLedgerClientBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, input_amount)
        .with_value(extra_policy.clone(), extra_amount)
        .finish_output()
        .build_in_memory();

    let contract = SmartContract::new(TransferADASmartContract, ledger_client);

    let call = Endpoint::Transfer {
        amount,
        recipient: alice.clone(),
    };

    contract.hit_endpoint(call).await.unwrap();

    let alice_expected = amount;
    let alice_actual = contract
        .ledger_client()
        .balance_at_address(&alice, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(alice_expected, alice_actual);

    let me_expected = input_amount - amount;
    let me_actual = contract
        .ledger_client()
        .balance_at_address(&me, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(me_expected, me_actual);

    let expected_extra_amount = extra_amount;
    let actual_extra_amount = contract
        .ledger_client()
        .balance_at_address(&me, &extra_policy)
        .await
        .unwrap();
    assert_eq!(expected_extra_amount, actual_extra_amount);
}
