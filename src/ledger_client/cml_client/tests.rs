use super::*;
use crate::{
    ledger_client::cml_client::{
        blockfrost_ledger::BlockFrostLedger,
        key_manager::{KeyManager, TESTNET},
    },
    PolicyId,
};
use blockfrost_http_client::load_key_from_file;
use cardano_multiplatform_lib::address::BaseAddress;
use std::time::Duration;
use test_helpers::{
    always_succeeds_script_address, claim_always_succeeds_datum_tx, lock_at_always_succeeds_tx,
    output_from_tx, transfer_tx,
};
use tokio::time::sleep;

mod test_helpers;

// const MAINNET_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";
const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

async fn get_test_client<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>() -> (
    CMLLedgerCLient<BlockFrostLedger, KeyManager, Datum, Redeemer>,
    BaseAddress,
) {
    let api_key = load_key_from_file(CONFIG_PATH).unwrap();
    let ledger = BlockFrostLedger::new(TEST_URL, &api_key);
    let keys = KeyManager::new(CONFIG_PATH.to_string(), TESTNET);
    let base_addr = keys.base_addr().await.unwrap();
    (CMLLedgerCLient::new(ledger, keys, TESTNET), base_addr)
}

#[ignore]
#[tokio::test]
async fn get_all_my_utxos() {
    let (client, base_addr) = get_test_client::<(), ()>().await;
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);
    let my_utxos = client.all_outputs_at_address(&my_addr).await.unwrap();

    dbg!(my_utxos);
}

#[ignore]
#[tokio::test]
async fn get_my_lovelace_balance() {
    let (client, base_addr) = get_test_client::<(), ()>().await;
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);
    let my_balance = client
        .balance_at_address(&my_addr, &PolicyId::ADA)
        .await
        .unwrap();

    println!();
    println!("ADA: {:?}", my_balance);
}

#[ignore]
#[tokio::test]
async fn get_my_native_token_balance() {
    let (client, base_addr) = get_test_client::<(), ()>().await;
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);
    let policy = PolicyId::native_token(
        "57fca08abbaddee36da742a839f7d83a7e1d2419f1507fcbf3916522",
        &None,
    );
    let my_balance = client.balance_at_address(&my_addr, &policy).await.unwrap();

    println!();
    println!("Native Token {:?}: {:?}", policy, my_balance);
}

#[ignore]
#[tokio::test]
async fn transfer_self_tx() {
    let (client, base_addr) = get_test_client::<(), ()>().await;
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);
    let transfer_amount = 6_000_000;
    let unbuilt_tx = transfer_tx(my_addr, transfer_amount);
    let res = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", res);
}

#[ignore]
#[tokio::test]
async fn create_datum_wait_and_then_redeem_same_datum() {
    let lock_amount = 6_000_000;
    let unbuilt_tx = lock_at_always_succeeds_tx(lock_amount);
    let (client, _) = get_test_client::<(), ()>().await;
    let tx_id = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", &tx_id);
    let script_addr = always_succeeds_script_address(TESTNET);

    let mut tries = 30;
    println!("Attempting to find and spend datum from {:?}", &tx_id);
    loop {
        println!("...");
        sleep(Duration::from_secs(5)).await;
        let script_outputs = client.all_outputs_at_address(&script_addr).await.unwrap();
        if let Some(my_output) = output_from_tx::<()>(&tx_id.as_str(), &script_outputs) {
            println!("Found UTxO");
            println!("Issuing redeeming tx");
            let unbuilt_tx = claim_always_succeeds_datum_tx(my_output);
            let res = client.issue(unbuilt_tx).await.unwrap();
            println!("{:?}", res);
            return;
        }
        tries -= 1;
        if tries < 0 {
            println!("Failed to find UTxO for {:?}", tx_id);
            return;
        }
    }
}
