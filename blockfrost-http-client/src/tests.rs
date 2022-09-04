use super::*;
use crate::keys::{my_base_addr, TESTNET};
use cardano_multiplatform_lib::address::RewardAddress;
use cardano_multiplatform_lib::builders::output_builder::SingleOutputBuilderResult;
use cardano_multiplatform_lib::builders::tx_builder::{
    TransactionBuilder, TransactionBuilderConfigBuilder,
};
use cardano_multiplatform_lib::crypto::TransactionHash;
use cardano_multiplatform_lib::ledger::alonzo::fees::LinearFee;
use cardano_multiplatform_lib::ledger::common::value::{BigInt, BigNum, Value};
use cardano_multiplatform_lib::plutus::{
    CostModel, Costmdls, ExUnitPrices, Language, PlutusData, PlutusScript, PlutusScriptEnum,
    PlutusV1Script,
};
use cardano_multiplatform_lib::{
    RequiredSigners, TransactionInput, TransactionOutput, UnitInterval,
};
use std::fs::File;
use std::io::Read;

use cardano_multiplatform_lib::address::Address as CMLAddress;
use cardano_multiplatform_lib::builders::input_builder::SingleInputBuilder;
use cardano_multiplatform_lib::builders::witness_builder::PartialPlutusWitness;
use serde_json::map::Values;

// Most of these values are made up
fn test_tx_builder() -> TransactionBuilder {
    let coefficient = BigNum::from_str("44").unwrap();
    let constant = BigNum::from_str("155381").unwrap();
    let linear_fee = LinearFee::new(&coefficient, &constant);
    let pool_deposit = BigNum::from_str("500000000").unwrap();
    let key_deposit = BigNum::from_str("2000000").unwrap();
    let coins_per_utxo_byte = BigNum::from_str("34482").unwrap();
    let mem_num = BigNum::from_str("123").unwrap();
    let mem_den = BigNum::from_str("456").unwrap();
    let mem_price = UnitInterval::new(&mem_num, &mem_den);
    let step_num = BigNum::from_str("123").unwrap();
    let step_den = BigNum::from_str("456").unwrap();
    let step_price = UnitInterval::new(&step_num, &step_den);
    let ex_unit_prices = ExUnitPrices::new(&mem_price, &step_price);
    let mut cost_models = Costmdls::new();
    let language = Language::new_plutus_v1();
    let op_costs = Vec::new();
    let v1_model = CostModel::new(&language, &op_costs);
    cost_models.insert(&v1_model);
    let tx_builder_cfg = TransactionBuilderConfigBuilder::new()
        .fee_algo(&linear_fee)
        .pool_deposit(&pool_deposit)
        .key_deposit(&key_deposit)
        .max_value_size(4000)
        .max_tx_size(8000)
        .coins_per_utxo_byte(&coins_per_utxo_byte)
        .ex_unit_prices(&ex_unit_prices)
        .collateral_percentage(1)
        .max_collateral_inputs(5)
        .costmdls(&cost_models)
        .build()
        .unwrap();
    TransactionBuilder::new(&tx_builder_cfg)
}

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

    let mut tx_builder = test_tx_builder();

    // Add input
    let index = BigNum::from_str("0").unwrap();
    let hash_raw = "8561258e210352fba2ac0488afed67b3427a27ccf1d41ec030c98a8199bc22ec";
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );
    let coin = BigNum::from_str("23000000").unwrap();
    let value = Value::new(&coin);
    let utxo_info = TransactionOutput::new(&base_addr.to_address(), &value);
    let input_builder = SingleInputBuilder::new(&input, &utxo_info);
    let script_bytes = read_script_from_file("./game.plutus");
    let v1 = PlutusV1Script::new(script_bytes);
    let script = PlutusScript::from_v1(&v1);
    let redeemer_integer = BigInt::from_str("456").unwrap();
    let data = PlutusData::new_integer(&redeemer_integer);
    let partial_witness = PartialPlutusWitness::new(&script, &data);
    let required_signers = RequiredSigners::new();
    let datum_integer = BigInt::from_str("123").unwrap();
    let datum = PlutusData::new_integer(&datum_integer);
    let input_builder_res = input_builder
        .plutus_script(&partial_witness, &required_signers, &datum)
        .unwrap();
    tx_builder.add_input(&input_builder_res);

    // Add output
    let output_address = CMLAddress::from_bech32("addr_test1qpu5vlrf4xkxv2qpwngf6cjhtw542ayty80v8dyr49rf5ewvxwdrt70qlcpeeagscasafhffqsxy36t90ldv06wqrk2qum8x5w").unwrap();
    let coin = BigNum::from_str("17758450").unwrap();
    let value = Value::new(&coin);
    let output = TransactionOutput::new(&output_address, &value);
    let single_output = SingleOutputBuilderResult::new(&output);
    tx_builder.add_output(&single_output).unwrap();

    // Add change address
    let change_address = CMLAddress::from_bech32(
        "addr_test1gz2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzerspqgpsqe70et",
    )
    .unwrap();
    tx_builder.add_change_if_needed(&change_address).unwrap();

    let tx_redeemer_builder = tx_builder.build().unwrap();
    let signed_tx_builder = tx_redeemer_builder.build().unwrap();

    let transaction = signed_tx_builder.build_unchecked();
    let bytes = transaction.to_bytes();

    let res = bf.execution_units(&bytes).await.unwrap();

    dbg!(res);
}

fn read_script_from_file(file_path: &str) -> Vec<u8> {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let script_file: PlutusScriptFile = serde_json::from_str(&data).unwrap();
    hex::decode(script_file.cbor_hex()).unwrap()
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct PlutusScriptFile {
    r#type: String,
    description: String,
    cborHex: String,
}

impl PlutusScriptFile {
    pub fn cbor_hex(&self) -> &str {
        &self.cborHex
    }
}

// -- create a data script for the guessing game by hashing the string
// -- and lifting the hash to its on-chain representation
// hashString :: Haskell.String -> HashedString
// hashString = HashedString . sha2_256 . toBuiltin . C.pack
//
// -- create a redeemer script for the guessing game by lifting the
// -- string to its on-chain representation
// clearString :: Haskell.String -> ClearString
// clearString = ClearString . toBuiltin . C.pack
//
// -- | The validation function (Datum -> Redeemer -> ScriptContext -> Bool)
// validateGuess :: HashedString -> ClearString -> ScriptContext -> Bool
// validateGuess hs cs _ = isGoodGuess hs cs
//
// isGoodGuess :: HashedString -> ClearString -> Bool
// isGoodGuess (HashedString actual) (ClearString guess') = actual == sha2_256 guess'
#[test]
fn can_submit_script() {}
