use super::*;
use naumachia::{
    ledger_client::test_ledger_client::TestLedgerClientBuilder,
    smart_contract::{
        SmartContract,
        SmartContractTrait,
    },
    Address,
    Network,
};

// Ignore because the game script is funky with Aiken
#[ignore]
#[tokio::test]
async fn lock_and_claim() {
    let me = Address::from_bech32("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm").unwrap();
    let start_amount = 100_000_000;
    let backend = TestLedgerClientBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let amount = 10_000_000;
    let secret = "my secret";
    let endpoint = GameEndpoints::Lock {
        amount,
        secret: secret.to_string(),
    };
    let script = get_script().unwrap();
    let contract = SmartContract::new(GameLogic, backend);
    let network = Network::Testnet;
    contract.hit_endpoint(endpoint).await.unwrap();
    {
        let expected = amount;
        let actual = contract
            .ledger_client()
            .balance_at_address(&script.address(network).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    {
        let expected = start_amount - amount;
        let actual = contract
            .ledger_client()
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
    let instance = contract
        .ledger_client()
        .all_outputs_at_address(&script.address(network).unwrap())
        .await
        .unwrap()
        .pop()
        .unwrap();
    let call = GameEndpoints::Guess {
        output_id: instance.id().clone(),
        guess: secret.to_string(),
    };

    contract.hit_endpoint(call).await.unwrap();
    {
        let actual = contract
            .ledger_client()
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(actual, start_amount);
    }
    {
        let script_balance = contract
            .ledger_client()
            .balance_at_address(&script.address(network).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(script_balance, 0);
    }
}
