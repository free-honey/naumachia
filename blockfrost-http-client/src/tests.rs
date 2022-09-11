use super::*;
use crate::keys::{my_base_addr, my_priv_key, TESTNET};
use cardano_multiplatform_lib::builders::output_builder::TransactionOutputBuilder;
use cardano_multiplatform_lib::builders::redeemer_builder::RedeemerWitnessKey;
use cardano_multiplatform_lib::builders::tx_builder::CoinSelectionStrategyCIP2;
use cardano_multiplatform_lib::builders::witness_builder::PlutusScriptWitness;
use cardano_multiplatform_lib::ledger::common::hash::hash_plutus_data;
use cardano_multiplatform_lib::plutus::{ExUnits, RedeemerTag};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    address::{EnterpriseAddress, RewardAddress, StakeCredential},
    builders::input_builder::{InputBuilderResult, SingleInputBuilder},
    builders::output_builder::SingleOutputBuilderResult,
    builders::tx_builder::{
        ChangeSelectionAlgo, TransactionBuilder, TransactionBuilderConfigBuilder,
    },
    builders::witness_builder::PartialPlutusWitness,
    crypto::TransactionHash,
    ledger::alonzo::fees::LinearFee,
    ledger::common::hash::hash_transaction,
    ledger::common::value::{BigNum, Int, Value as CMLValue},
    ledger::shelley::witness::make_vkey_witness,
    plutus::{
        CostModel, Costmdls, ExUnitPrices, Language, PlutusData, PlutusScript, PlutusV1Script,
    },
    AssetName, Assets, Datum, MultiAsset, PolicyID, RequiredSigners, TransactionInput,
    TransactionOutput, UnitInterval,
};
use std::fs::File;
use std::io::Read;

#[ignore]
#[tokio::test]
async fn genesis() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let res = bf.genesis().await.unwrap();
    println!("{:?}", res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn protocol_params() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let epoch = 227;
    let res = bf.protocol_params(epoch).await.unwrap();
    dbg!("{:?}", res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn my_utxos() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let address = my_base_addr().to_address().to_bech32(None).unwrap();
    let res = bf.utxos(&address).await.unwrap();
    dbg!(&res);
    Ok(())
}

#[ignore]
#[tokio::test]
async fn script_utxos() -> Result<()> {
    let bf = get_test_bf_http_client().unwrap();
    let address = always_succeeds_script_address(TESTNET)
        .to_bech32(None)
        .unwrap();
    let filtered: Vec<_> = bf
        .utxos(&address)
        .await
        .unwrap()
        .into_iter()
        // .filter(|utxo| utxo.amount()[0].quantity == "2000000")
        .filter(|utxo| {
            (utxo.tx_hash() == "8a2481f461be07a6bdde32c23b3915b896f43e13f96902f3a225ea3bfe2fc1aa")
                | (utxo.tx_hash()
                    == "b8df1678952403cc33c6a8e41b1c6b94ac91cc629b14e8ede9448e6f95b625a8")
                | (utxo.tx_hash()
                    == "d5be9549bfb82b5981f6cdf49187b6140bac5f129adbb50281ee0e680c0a411a")
        })
        .collect();
    let mut res = Vec::new();
    for utxo in filtered {
        if let Some(datum_hash) = utxo.data_hash() {
            let datum = bf.datum(&datum_hash).await.unwrap();
            let new_utxo = utxo.with_data(Some(datum));
            res.push(new_utxo);
        }
    }
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
    let address = always_succeeds_script_address(TESTNET)
        .to_bech32(None)
        .unwrap();
    let res = bf.address_info(&address).await.unwrap();
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

// Most of these values are made up
fn test_tx_builder() -> TransactionBuilder {
    let coefficient = BigNum::from_str("44").unwrap();
    let constant = BigNum::from_str("155381").unwrap();
    let linear_fee = LinearFee::new(&coefficient, &constant);

    let pool_deposit = BigNum::from_str("500000000").unwrap();
    let key_deposit = BigNum::from_str("2000000").unwrap();

    let coins_per_utxo_byte = BigNum::from_str("4310").unwrap();
    let mem_num = BigNum::from_str("577").unwrap();
    let mem_den = BigNum::from_str("10000").unwrap();
    let mem_price = UnitInterval::new(&mem_num, &mem_den);
    let step_num = BigNum::from_str("721").unwrap();
    let step_den = BigNum::from_str("10000000").unwrap();
    let step_price = UnitInterval::new(&step_num, &step_den);
    let ex_unit_prices = ExUnitPrices::new(&mem_price, &step_price);
    let vasil_v1_cost_models = vec![
        205665, 812, 1, 1, 1000, 571, 0, 1, 1000, 24177, 4, 1, 1000, 32, 117366, 10475, 4, 23000,
        100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 23000, 100, 100, 100, 23000, 100,
        19537, 32, 175354, 32, 46417, 4, 221973, 511, 0, 1, 89141, 32, 497525, 14068, 4, 2, 196500,
        453240, 220, 0, 1, 1, 1000, 28662, 4, 2, 245000, 216773, 62, 1, 1060367, 12586, 1, 208512,
        421, 1, 187000, 1000, 52998, 1, 80436, 32, 43249, 32, 1000, 32, 80556, 1, 57667, 4, 1000,
        10, 197145, 156, 1, 197145, 156, 1, 204924, 473, 1, 208896, 511, 1, 52467, 32, 64832, 32,
        65493, 32, 22558, 32, 16563, 32, 76511, 32, 196500, 453240, 220, 0, 1, 1, 69522, 11687, 0,
        1, 60091, 32, 196500, 453240, 220, 0, 1, 1, 196500, 453240, 220, 0, 1, 1, 806990, 30482, 4,
        1927926, 82523, 4, 265318, 0, 4, 0, 85931, 32, 205665, 812, 1, 1, 41182, 32, 212342, 32,
        31220, 32, 32696, 32, 43357, 32, 32247, 32, 38314, 32, 9462713, 1021, 10,
    ];
    let cm = CostModel::new(
        &Language::new_plutus_v1(),
        &vasil_v1_cost_models.iter().map(|&i| Int::from(i)).collect(),
    );
    let mut cost_models = Costmdls::new();
    cost_models.insert(&cm);

    let tx_builder_cfg = TransactionBuilderConfigBuilder::new()
        .fee_algo(&linear_fee)
        .pool_deposit(&pool_deposit)
        .key_deposit(&key_deposit)
        .max_value_size(5000)
        .max_tx_size(16384)
        .coins_per_utxo_byte(&coins_per_utxo_byte)
        .ex_unit_prices(&ex_unit_prices)
        .collateral_percentage(150)
        .max_collateral_inputs(3)
        .costmdls(&cost_models)
        .build()
        .unwrap();
    TransactionBuilder::new(&tx_builder_cfg)
}

fn read_script_from_file(file_path: &str) -> PlutusScriptFile {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).unwrap()
}

fn always_succeeds_script() -> PlutusScript {
    let script_file = read_script_from_file("./always-succeeds-spending.plutus");
    let script_hex = script_file.cborHex;
    let script_bytes = hex::decode(&script_hex).unwrap();
    let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap();
    PlutusScript::from_v1(&v1)
}

fn always_succeeds_script_address(network: u8) -> CMLAddress {
    let script = always_succeeds_script();
    let script_hash = script.hash();
    let stake_cred = StakeCredential::from_scripthash(&script_hash);
    let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
    enterprise_addr.to_address()
}

fn always_succeeds_script_input(amt: u64, hash_raw: &str, index: u64) -> InputBuilderResult {
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let script_input = TransactionInput::new(
        &tx_hash,      // tx hash
        &index.into(), // index
    );

    let script = always_succeeds_script();
    let script_addr = always_succeeds_script_address(TESTNET);

    let coin = amt.into();
    let value = CMLValue::new(&coin);
    let utxo_info = TransactionOutput::new(&script_addr, &value);
    let input_builder = SingleInputBuilder::new(&script_input, &utxo_info);

    let data = PlutusData::new_bytes(Vec::new());
    let script_witness = PlutusScriptWitness::from_script(script);
    let partial_witness = PartialPlutusWitness::new(&script_witness, &data);

    let required_signers = RequiredSigners::new();
    let datum = PlutusData::new_bytes(Vec::new());
    input_builder
        .plutus_script(&partial_witness, &required_signers, &datum)
        .unwrap()
}

fn input_from_utxo(my_address: &CMLAddress, utxo: &UTxO) -> InputBuilderResult {
    let index = utxo.output_index().into();
    let hash_raw = utxo.tx_hash();
    let tx_hash = TransactionHash::from_hex(hash_raw).unwrap();
    let payment_input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );
    let value = cmlvalue_from_values(&utxo.amount());
    let utxo_info = TransactionOutput::new(&my_address, &value);
    let input_builder = SingleInputBuilder::new(&payment_input, &utxo_info);

    input_builder.payment_key().unwrap()
}

fn cmlvalue_from_values(values: &[Value]) -> CMLValue {
    let mut cml_value = CMLValue::zero();
    for value in values.iter() {
        let Value { unit, quantity } = value;
        let add_value = match unit.as_str() {
            "lovelace" => CMLValue::new(&BigNum::from_str(quantity).unwrap()),
            _ => {
                let policy_id_hex = &unit[..56];
                let policy_id = PolicyID::from_hex(policy_id_hex).unwrap();
                let asset_name_hex = &unit[56..];
                let asset_name_bytes = hex::decode(asset_name_hex).unwrap();
                let asset_name = AssetName::new(asset_name_bytes.into()).unwrap();
                let mut assets = Assets::new();
                assets.insert(&asset_name, &BigNum::from_str(quantity).unwrap());
                let mut multi_assets = MultiAsset::new();
                multi_assets.insert(&policy_id, &assets);
                CMLValue::new_from_assets(&multi_assets)
            }
        };
        cml_value = cml_value.checked_add(&add_value).unwrap();
    }
    cml_value
}

use crate::schemas::Value;
use serde::Deserialize;

#[allow(non_snake_case)]
#[allow(unused)]
#[derive(Deserialize, Debug)]
struct PlutusScriptFile {
    r#type: String,
    description: String,
    cborHex: String,
}

impl PlutusScriptFile {
    pub fn _cbor_hex(&self) -> &str {
        &self.cborHex
    }
}

#[ignore]
#[tokio::test]
async fn send_to_self() {
    let my_base_addr = my_base_addr();
    let my_address = my_base_addr.to_address();
    let priv_key = my_priv_key();

    let bf = get_test_bf_http_client().unwrap();

    let my_utxos = bf
        .utxos(&my_address.to_bech32(None).unwrap())
        .await
        .unwrap();

    let mut tx_builder = test_tx_builder();

    for utxo in my_utxos.iter() {
        let input = input_from_utxo(&my_address, utxo);
        tx_builder.add_utxo(&input);
    }

    let coin = 150_000_000.into();
    let value = CMLValue::new(&coin);
    let output = TransactionOutput::new(&my_address, &value);
    let res = SingleOutputBuilderResult::new(&output);

    tx_builder.add_output(&res).unwrap();

    let strat = CoinSelectionStrategyCIP2::LargestFirstMultiAsset;
    tx_builder.select_utxos(strat).unwrap();

    let algo = ChangeSelectionAlgo::Default;
    let mut signed_tx_builder = tx_builder.build(algo, &my_address).unwrap();
    let unchecked_tx = signed_tx_builder.build_unchecked();
    let tx_body = unchecked_tx.body();
    let tx_hash = hash_transaction(&tx_body);
    dbg!(tx_hash.to_hex());
    let vkey_witness = make_vkey_witness(&tx_hash, &priv_key);
    signed_tx_builder.add_vkey(&vkey_witness);
    let tx = signed_tx_builder.build_checked().unwrap();
    println!("{}", tx.to_json().unwrap());
    let submit_res = bf.submit_tx(&tx.to_bytes()).await.unwrap();
    dbg!(&submit_res);
}

#[ignore]
#[tokio::test]
async fn init_always_succeeds_contract() {
    let my_base_addr = my_base_addr();
    let my_address = my_base_addr.to_address();
    let priv_key = my_priv_key();

    let bf = get_test_bf_http_client().unwrap();

    let my_utxos = bf
        .utxos(&my_address.to_bech32(None).unwrap())
        .await
        .unwrap();

    let mut tx_builder = test_tx_builder();

    for utxo in my_utxos.iter() {
        let input = input_from_utxo(&my_address, utxo);
        tx_builder.add_utxo(&input);
    }

    let script_address = always_succeeds_script_address(TESTNET);
    let coin = 2_000_000.into();
    let value = CMLValue::new(&coin);
    let mut output = TransactionOutput::new(&script_address, &value);
    let empty_data = PlutusData::new_bytes(Vec::new());
    let data_hash = hash_plutus_data(&empty_data);
    let datum = Datum::new_data_hash(&data_hash);
    output.set_datum(&datum);
    let mut res = SingleOutputBuilderResult::new(&output);
    res.set_communication_datum(&empty_data);

    tx_builder.add_output(&res).unwrap();

    let strat = CoinSelectionStrategyCIP2::LargestFirstMultiAsset;
    tx_builder.select_utxos(strat).unwrap();

    let algo = ChangeSelectionAlgo::Default;
    let mut signed_tx_builder = tx_builder.build(algo, &my_address).unwrap();

    let unchecked_tx = signed_tx_builder.build_unchecked();
    let tx_body = unchecked_tx.body();
    let tx_hash = hash_transaction(&tx_body);
    dbg!(tx_hash.to_hex());
    let vkey_witness = make_vkey_witness(&tx_hash, &priv_key);
    signed_tx_builder.add_vkey(&vkey_witness);
    let tx = signed_tx_builder.build_checked().unwrap();
    println!("{}", tx.to_json().unwrap());
    let submit_res = bf.submit_tx(&tx.to_bytes()).await.unwrap();
    // let submit_res = bf.execution_units(&tx.to_bytes()).await.unwrap();
    dbg!(&submit_res);
}

#[ignore]
#[tokio::test]
async fn spend_datum() {
    let my_base_addr = my_base_addr();
    let my_address = my_base_addr.to_address();
    let priv_key = my_priv_key();

    let bf = get_test_bf_http_client().unwrap();

    let my_utxos = bf
        .utxos(&my_address.to_bech32(None).unwrap())
        .await
        .unwrap();

    let mut tx_builder = test_tx_builder();

    let hash_raw = "d5be9549bfb82b5981f6cdf49187b6140bac5f129adbb50281ee0e680c0a411a";
    let index = 0;
    let script_input = always_succeeds_script_input(2_000_000, hash_raw, index);
    tx_builder.add_input(&script_input).unwrap();

    for utxo in my_utxos.iter() {
        let input = input_from_utxo(&my_address, utxo);
        tx_builder.add_utxo(&input);
    }

    let input_utxo = TransactionOutputBuilder::new()
        .with_address(&my_address)
        .next()
        .unwrap()
        .with_coin(&1_000_000u64.into())
        .build()
        .unwrap();
    let collateral_intput = SingleInputBuilder::new(
        &TransactionInput::new(
            &TransactionHash::from_hex(
                "862cc35a4a90059a2c5d55c3890961a368707bca66eaf27ea2a4862883f80df2",
            )
            .unwrap(),
            &0.into(),
        ),
        &input_utxo.output(),
    )
    .payment_key()
    .unwrap();

    tx_builder.add_collateral(&collateral_intput).unwrap();

    let strat = CoinSelectionStrategyCIP2::LargestFirstMultiAsset;
    tx_builder.select_utxos(strat).unwrap();

    let algo = ChangeSelectionAlgo::Default;
    // let signed_tx_builder = tx_builder.build(algo, &change_address).unwrap();
    let tx_redeemer_builder = tx_builder.build_for_evaluation(algo, &my_address).unwrap();
    let transaction = tx_redeemer_builder.draft_tx();

    let bytes = transaction.to_bytes();

    let res = bf.execution_units(&bytes).await.unwrap();

    let spend = res.get_spend().unwrap();

    tx_builder.set_exunits(
        &RedeemerWitnessKey::new(&RedeemerTag::new_spend(), &BigNum::from(2)), // TODO: How do I know which index?
        &ExUnits::new(&spend.memory().into(), &spend.steps().into()),
    );

    let algo = ChangeSelectionAlgo::Default;
    let mut signed_tx_builder = tx_builder.build(algo, &my_address).unwrap();
    let unchecked_tx = signed_tx_builder.build_unchecked();

    let tx_body = unchecked_tx.body();
    let tx_hash = hash_transaction(&tx_body);
    dbg!(tx_hash.to_hex());
    let vkey_witness = make_vkey_witness(&tx_hash, &priv_key);
    signed_tx_builder.add_vkey(&vkey_witness);
    let tx = signed_tx_builder.build_checked().unwrap();
    println!("{}", tx.to_json().unwrap());
    let submit_res = bf.submit_tx(&tx.to_bytes()).await.unwrap();
    dbg!(&submit_res);
}
