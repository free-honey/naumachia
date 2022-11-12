use super::*;
use naumachia::address::Address;
use naumachia::ledger_client::in_memory_ledger::TestBackendsBuilder;
use naumachia::smart_contract::{SmartContract, SmartContractTrait};

#[tokio::test]
async fn lock_and_claim() {
    let me = Address::new("me");
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::ADA, start_amount)
        .finish_output()
        .build();

    let amount = 10_000_000;
    let secret = "my secret";
    let endpoint = GameEndpoints::Lock {
        amount,
        secret: secret.to_string(),
    };
    let script = get_script().unwrap();
    let contract = SmartContract::new(&GameLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
    {
        let expected = amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    {
        let expected = start_amount - amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
    let instance = backend
        .ledger_client
        .all_outputs_at_address(&script.address(0).unwrap())
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
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(actual, start_amount);
    }
    {
        let script_balance = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(script_balance, 0);
    }
}
