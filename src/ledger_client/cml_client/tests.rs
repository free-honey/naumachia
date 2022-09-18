use super::*;
use crate::ledger_client::cml_client::blockfrost_ledger::BlockFrostLedger;
use crate::ledger_client::cml_client::key_manager::{KeyManager, TESTNET};
use crate::output::UnbuiltOutput;
use crate::PolicyId;
use blockfrost_http_client::{keys::my_base_addr, load_key_from_file};

// const MAINNET_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";
const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

fn get_test_client<Datum, Redeemer>(
) -> CMLLedgerCLient<BlockFrostLedger, KeyManager, Datum, Redeemer> {
    let api_key = load_key_from_file(CONFIG_PATH).unwrap();
    let ledger = BlockFrostLedger::new(TEST_URL, &api_key);
    let keys = KeyManager::new(CONFIG_PATH.to_string(), TESTNET);
    CMLLedgerCLient::new(ledger, keys)
}

#[ignore]
#[tokio::test]
async fn get_all_my_utxos() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_client::<(), ()>();

    let my_utxos = client.outputs_at_address(&my_addr).await.unwrap();

    dbg!(my_utxos);
}

#[ignore]
#[tokio::test]
async fn get_my_lovelace_balance() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_client::<(), ()>();

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
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);

    let client = get_test_client::<(), ()>();

    let policy = PolicyId::native_token("57fca08abbaddee36da742a839f7d83a7e1d2419f1507fcbf3916522");
    let my_balance = client.balance_at_address(&my_addr, &policy).await.unwrap();

    println!();
    println!("Native Token {:?}: {:?}", policy, my_balance);
}

fn transfer_tx(recipient: Address, amount: u64) -> UnbuiltTransaction<(), ()> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let output = UnbuiltOutput::new_wallet(recipient, values);
    UnbuiltTransaction {
        script_inputs: vec![],
        unbuilt_outputs: vec![output],
        redeemers: vec![],
        validators: Default::default(),
        minting: Default::default(),
        policies: Default::default(),
    }
}

#[ignore]
#[tokio::test]
async fn transfer_self_tx() {
    let base_addr = my_base_addr();
    let addr_string = base_addr.to_address().to_bech32(None).unwrap();
    let my_addr = Address::Base(addr_string);
    let transfer_amount = 2_000_000;
    let unbuilt_tx = transfer_tx(my_addr, transfer_amount);
    let client = get_test_client::<(), ()>();
    let res = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", res);
}
