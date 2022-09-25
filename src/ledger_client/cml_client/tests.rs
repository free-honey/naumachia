use super::*;
use crate::output::OutputId;
use crate::scripts::{ScriptResult, TxContext, ValidatorCode};
use crate::{
    ledger_client::cml_client::{
        blockfrost_ledger::BlockFrostLedger,
        key_manager::{KeyManager, TESTNET},
    },
    output::UnbuiltOutput,
    values::Values,
    PolicyId,
};
use blockfrost_http_client::{keys::my_base_addr, load_key_from_file};
use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::builders::witness_builder::{
    PartialPlutusWitness, PlutusScriptWitness,
};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script};
use cardano_multiplatform_lib::Transaction;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

// const MAINNET_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";
const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

fn get_test_client<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
) -> CMLLedgerCLient<BlockFrostLedger, KeyManager, Datum, Redeemer> {
    let api_key = load_key_from_file(CONFIG_PATH).unwrap();
    let ledger = BlockFrostLedger::new(TEST_URL, &api_key);
    let keys = KeyManager::new(CONFIG_PATH.to_string(), TESTNET);
    CMLLedgerCLient::new(ledger, keys, TESTNET)
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

    let policy = PolicyId::native_token(
        "57fca08abbaddee36da742a839f7d83a7e1d2419f1507fcbf3916522",
        &None,
    );
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
    let transfer_amount = 6_000_000;
    let unbuilt_tx = transfer_tx(my_addr, transfer_amount);
    let client = get_test_client::<(), ()>();
    let res = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", res);
}

fn lock_at_always_succeeds_tx(amount: u64) -> UnbuiltTransaction<(), ()> {
    let script_address = always_succeeds_script_address(TESTNET);
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let datum = ();
    let output = UnbuiltOutput::new_validator(script_address, values, datum);
    UnbuiltTransaction {
        script_inputs: vec![],
        unbuilt_outputs: vec![output],
        minting: Default::default(),
        policies: Default::default(),
    }
}

fn always_succeeds_script_cml_address(network: u8) -> CMLAddress {
    let script = always_succeeds_script();
    let script_hash = script.hash();
    let stake_cred = StakeCredential::from_scripthash(&script_hash);
    let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
    enterprise_addr.to_address()
}

fn always_succeeds_script_address(network: u8) -> Address {
    let cml_script_address = always_succeeds_script_cml_address(network);
    let script_address_str = cml_script_address.to_bech32(None).unwrap();
    Address::Script(script_address_str)
}

fn always_succeeds_hex() -> String {
    let script_file = read_script_from_file("./plutus/always-succeeds-spending.plutus");
    script_file.cborHex
}

fn always_succeeds_script() -> PlutusScript {
    let script_file = read_script_from_file("./plutus/always-succeeds-spending.plutus");
    let script_hex = script_file.cborHex;
    let script_bytes = hex::decode(&script_hex).unwrap();
    let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap();
    PlutusScript::from_v1(&v1)
}

fn read_script_from_file(file_path: &str) -> PlutusScriptFile {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).unwrap()
}

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Deserialize, Debug)]
struct PlutusScriptFile {
    r#type: String,
    description: String,
    cborHex: String,
}

#[ignore]
#[tokio::test]
async fn init_always_succeeds_script_tx() {
    let lock_amount = 6_000_000;
    let unbuilt_tx = lock_at_always_succeeds_tx(lock_amount);
    let client = get_test_client::<(), ()>();
    let res = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", res);
}

struct CMLValidator {
    script_hex: String,
    cml_script: PlutusScript,
}

impl CMLValidator {
    pub fn new_v1(script_hex: String) -> Self {
        let script_bytes = hex::decode(&script_hex).unwrap(); // TODO
        let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap(); // TODO
        let cml_script = PlutusScript::from_v1(&v1);
        CMLValidator {
            script_hex,
            cml_script,
        }
    }
}

impl ValidatorCode<(), ()> for CMLValidator {
    fn execute(&self, datum: (), redeemer: (), ctx: TxContext) -> ScriptResult<()> {
        todo!()
    }

    fn address(&self) -> Address {
        let network = TESTNET;
        let script_hash = self.cml_script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();
        let script_address_str = cml_script_address.to_bech32(None).unwrap();
        Address::Script(script_address_str)
    }

    fn script_hex(&self) -> &str {
        &self.script_hex
    }
}

fn claim_always_succeeds_datum_tx(script_input: Output<()>) -> UnbuiltTransaction<(), ()> {
    let script = CMLValidator::new_v1(always_succeeds_hex());
    let script = Box::new(script) as Box<dyn ValidatorCode<(), ()>>;
    UnbuiltTransaction {
        script_inputs: vec![(script_input.clone(), (), script)],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        policies: Default::default(),
    }
}

fn output_from_tx<D>(tx_id: &str, outputs: Vec<Output<D>>) -> Option<Output<D>> {
    for output in outputs {
        let id = output.id();
        let tx_hash = id.tx_hash();
        if tx_hash == tx_id {
            return Some(output);
        }
    }
    None
}

#[ignore]
#[tokio::test]
async fn spend_datum_tx() {
    let client = get_test_client::<(), ()>();
    let script_addr = always_succeeds_script_address(TESTNET);
    let script_outputs = client.outputs_at_address(&script_addr).await.unwrap();
    let tx_id = "abe7732220fe2fd0c0be8212b94b7197d72d4cc4d05203fbb3c8d3fd1113f3da";
    let my_output = output_from_tx::<()>(tx_id, script_outputs).unwrap();

    let unbuilt_tx = claim_always_succeeds_datum_tx(my_output);
    let res = client.issue(unbuilt_tx).await.unwrap();
    println!("{:?}", res);
}
