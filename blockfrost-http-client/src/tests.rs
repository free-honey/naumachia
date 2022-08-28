use super::*;
use cardano_multiplatform_lib::address::RewardAddress;

#[ignore]
#[tokio::test]
async fn genesis() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let _res = bf.genesis().await.unwrap();
    Ok(())
}

#[ignore]
#[tokio::test]
async fn utxos() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // TODO: Find a good stable address to use
    // let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";
    // let address = "addr_test1wrsexavz37208qda7mwwu4k7hcpg26cz0ce86f5e9kul3hqzlh22t";
    let address = "addr_test1wp9m8xkpt2tmy7madqldspgzgug8f2p3pwhz589cq75685slenwf4";
    let res = bf.utxos(address).await.unwrap();
    dbg!(&res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn datum() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // TODO: Find a good stable address to use
    // let datum_hash = "d1cede40100329bfd7edbb1245a4d24de23924f00341886dc5f5bf6d06c65629";
    let datum_hash = "a9fbe52ace8f89e0ae64d88f879e159b97d51f27d8f932c9aa165e5ce5f0f28e";
    let res = bf.datum(datum_hash).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
    Ok(())
}

#[ignore]
#[tokio::test]
async fn address_info() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    // let address = "addr1q97dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fsmqn6vh";
    // let address = "addr1qp7dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fs2k6a72";
    let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";

    let res = bf.address_info(address).await.unwrap();
    dbg!(&res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn account_associated_addresses() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();
    let staking_cred = base_addr.stake_cred();

    let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
        .to_address()
        .to_bech32(None)
        .unwrap();
    let res = bf.assoc_addresses(&reward_addr).await.unwrap();
    dbg!(&res);
}

#[ignore]
#[tokio::test]
async fn account_associated_addresses_total() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();
    let staking_cred = base_addr.stake_cred();

    let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
        .to_address()
        .to_bech32(None)
        .unwrap();
    let res = bf
        .account_associated_addresses_total(&reward_addr)
        .await
        .unwrap();
    dbg!(&res);
}

#[ignore]
#[tokio::test]
async fn execution_units() {
    let bf = get_test_bf_http_client().unwrap();
    let base_addr = my_base_addr();
    todo!()
}
