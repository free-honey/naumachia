use crate::{
    checking_account_validator, scripts::pull_validator::pull_validator, spend_token_policy,
    AllowedPuller, CheckingAccount, CheckingAccountEndpoints, CheckingAccountLogic,
    CHECKING_ACCOUNT_NFT_ASSET_NAME, SPEND_TOKEN_ASSET_NAME,
};
use naumachia::{
    address::PolicyId,
    ledger_client::test_ledger_client::TestBackendsBuilder,
    ledger_client::LedgerClient,
    scripts::context::pub_key_hash_from_address_if_available,
    scripts::{MintingPolicy, ValidatorCode},
    smart_contract::{SmartContract, SmartContractTrait},
    Address,
};

const NETWORK: u8 = 0;

#[tokio::test]
async fn init_creates_instance_with_correct_balance() {
    // given
    let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;

    // When
    let endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();

    // Then
    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(value, account_amount);
    let nft = script_output.values().as_iter().find(|(policy_id, amt)| {
        policy_id.asset_name() == Some(CHECKING_ACCOUNT_NFT_ASSET_NAME.to_string()) && **amt == 1
    });
    assert!(nft.is_some());
}

#[tokio::test]
async fn add_puller_creates_new_datum_for_puller() {
    let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
    let nft_id = vec![1, 2, 3, 4, 5];
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let puller = Address::from_bech32("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr").unwrap();
    let puller_pubkey_hash = pub_key_hash_from_address_if_available(&puller).unwrap();
    let endpoint = CheckingAccountEndpoints::AddPuller {
        checking_account_nft: hex::encode(&nft_id),
        puller: puller_pubkey_hash,
        amount_lovelace: 15_000_000,
        period: 1000,
        next_pull: 0,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
    let script = pull_validator().unwrap();
    let script_address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&script_address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();

    let parameterized_spending_token_policy = spend_token_policy().unwrap();
    let my_pubkey_hash = pub_key_hash_from_address_if_available(&me).unwrap();
    let policy = parameterized_spending_token_policy
        .apply(nft_id.into())
        .unwrap()
        .apply(my_pubkey_hash.into())
        .unwrap();
    let id = policy.id().unwrap();
    let value = script_output
        .values()
        .get(&PolicyId::NativeToken(
            id,
            Some(SPEND_TOKEN_ASSET_NAME.to_string()),
        ))
        .unwrap();
    assert_eq!(value, 1);
}

#[tokio::test]
async fn remove_puller__removes_the_allowed_puller() {
    let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
    let start_amount = 100_000_000;
    let nft_id = vec![1, 2, 3, 4, 5];
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let puller = Address::from_bech32("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr").unwrap();
    let puller_pubkey_hash = pub_key_hash_from_address_if_available(&puller).unwrap();
    let add_endpoint = CheckingAccountEndpoints::AddPuller {
        checking_account_nft: hex::encode(nft_id),
        puller: puller_pubkey_hash,
        amount_lovelace: 15_000_000,
        period: 1000,
        next_pull: 0,
    };

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(add_endpoint).await.unwrap();
    let script = pull_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let remove_endpoint = CheckingAccountEndpoints::RemovePuller { output_id };

    contract.hit_endpoint(remove_endpoint).await.unwrap();

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    assert!(outputs_at_address.pop().is_none());
}

#[tokio::test]
async fn fund_account__replaces_existing_balance_with_updated_amount() {
    let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;
    let fund_amount = 5_000_000;
    let init_endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(init_endpoint).await.unwrap();

    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let fund_endpoint = CheckingAccountEndpoints::FundAccount {
        output_id,
        fund_amount,
    };

    contract.hit_endpoint(fund_endpoint).await.unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(value, account_amount + fund_amount);
}

#[tokio::test]
async fn withdraw_from_account__replaces_existing_balance_with_updated_amount() {
    let me = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let account_amount = 10_000_000;
    let withdraw_amount = 5_000_000;
    let init_endpoint = CheckingAccountEndpoints::InitAccount {
        starting_lovelace: account_amount,
    };
    let contract = SmartContract::new(&CheckingAccountLogic, &backend);
    contract.hit_endpoint(init_endpoint).await.unwrap();

    let script = checking_account_validator().unwrap();
    let address = script.address(NETWORK).unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let output_id = script_output.id().to_owned();

    let fund_endpoint = CheckingAccountEndpoints::WithdrawFromAccount {
        output_id,
        withdraw_amount,
    };

    contract.hit_endpoint(fund_endpoint).await.unwrap();
    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(value, account_amount - withdraw_amount);
}

#[tokio::test]
async fn pull_from_account__replaces_existing_balances_with_updated_amounts() {
    let owner_address = Address::from_bech32("addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm").unwrap();
    let owner_pubkey_hash = pub_key_hash_from_address_if_available(&owner_address).unwrap();
    let puller = Address::from_bech32("addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr").unwrap();

    let allow_puller_script = pull_validator().unwrap();
    let allow_puller_address = allow_puller_script.address(NETWORK).unwrap();
    let account_script = checking_account_validator().unwrap();
    let spending_token_policy = vec![5, 5, 5, 5, 5];
    let account_address = account_script.address(NETWORK).unwrap();

    let account_amount = 100_000_000;
    let pull_amount = 15_000_000;
    let account_datum = CheckingAccount {
        owner: owner_pubkey_hash.clone(),
        spend_token_policy: spending_token_policy.clone(),
    }
    .into();
    let checking_account_nft_id = vec![1, 2, 3, 4, 5];
    let puller_pubkey_hash = pub_key_hash_from_address_if_available(&puller).unwrap();
    let allow_puller_datum = AllowedPuller {
        owner: owner_pubkey_hash,
        puller: puller_pubkey_hash,
        amount_lovelace: pull_amount,
        next_pull: 0,
        period: 0,
        spending_token: spending_token_policy.clone(),
        checking_account_nft: checking_account_nft_id.clone(),
    }
    .into();
    let backend = TestBackendsBuilder::new(&puller)
        .start_output(&account_address)
        .with_datum(account_datum)
        .with_value(PolicyId::Lovelace, account_amount)
        .with_value(
            PolicyId::NativeToken(hex::encode(&checking_account_nft_id), None),
            1,
        )
        .finish_output()
        .start_output(&allow_puller_address)
        .with_datum(allow_puller_datum)
        .with_value(
            PolicyId::NativeToken(hex::encode(spending_token_policy), None),
            1,
        )
        .finish_output()
        .build_in_memory();

    let contract = SmartContract::new(&CheckingAccountLogic, &backend);

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&account_address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let checking_account_output_id = script_output.id().to_owned();

    let mut outputs_at_address = backend
        .ledger_client
        .all_outputs_at_address(&allow_puller_address)
        .await
        .unwrap();
    let script_output = outputs_at_address.pop().unwrap();
    let allow_pull_output_id = script_output.id().to_owned();

    // When
    let pull_endpoint = CheckingAccountEndpoints::PullFromCheckingAccount {
        allow_pull_output_id,
        checking_account_output_id,
        amount: pull_amount,
    };
    contract.hit_endpoint(pull_endpoint).await.unwrap();

    // Then
    let mut outputs_at_account_address = backend
        .ledger_client
        .all_outputs_at_address(&account_address)
        .await
        .unwrap();
    let script_output = outputs_at_account_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(value, account_amount - pull_amount);

    let mut outputs_at_puller_address = backend
        .ledger_client
        .all_outputs_at_address(&puller)
        .await
        .unwrap();
    let script_output = outputs_at_puller_address.pop().unwrap();
    let value = script_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(value, pull_amount);
}
