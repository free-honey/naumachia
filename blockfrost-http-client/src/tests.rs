use super::*;
use crate::keys::{my_base_addr, TESTNET};
use cardano_multiplatform_lib::address::RewardAddress;
use cardano_multiplatform_lib::builders::tx_builder::{
    TransactionBuilder, TransactionBuilderConfigBuilder,
};
use cardano_multiplatform_lib::ledger::alonzo::fees::LinearFee;
use cardano_multiplatform_lib::ledger::common::value::BigNum;
use cardano_multiplatform_lib::plutus::ExUnitPrices;
use cardano_multiplatform_lib::{Transaction, UnitInterval};

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
    let fee = BigNum::zero();
    tx_builder.set_fee(&fee);
    let tx_redeemer_builder = tx_builder.build().unwrap();
    let signed_tx_builder = tx_redeemer_builder.build().unwrap();
    let transaction = signed_tx_builder.build_unchecked();
    let bytes = transaction.to_bytes();
    dbg!(&bytes);
}
