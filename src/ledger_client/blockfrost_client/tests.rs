use super::*;
use crate::PolicyId;
use blockfrost_http_client::{get_test_bf_http_client, keys::my_base_addr};

#[ignore]
#[tokio::test]
async fn get_all_my_utxos() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_bf_http_client().unwrap();

    let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

    let my_utxos = bf.outputs_at_address(&my_addr).await.unwrap();

    dbg!(my_utxos);
}

#[ignore]
#[tokio::test]
async fn get_my_lovelace_balance() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_bf_http_client().unwrap();

    let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

    let my_balance = bf
        .balance_at_address(&my_addr, &PolicyId::ADA)
        .await
        .unwrap();

    println!();
    println!("ADA: {:?}", my_balance);
}

#[ignore]
#[tokio::test]
async fn get_my_native_token_balance() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_bf_http_client().unwrap();

    let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

    let policy = PolicyId::native_token("57fca08abbaddee36da742a839f7d83a7e1d2419f1507fcbf3916522");
    let my_balance = bf.balance_at_address(&my_addr, &policy).await.unwrap();

    println!();
    println!("Native Token {:?}: {:?}", policy, my_balance);
}
