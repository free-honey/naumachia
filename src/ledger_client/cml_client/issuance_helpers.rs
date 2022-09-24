use super::error::*;
use crate::ledger_client::cml_client::UTxO;
use crate::output::Output;
use crate::values::Values;
use crate::{Address, PolicyId};
use blockfrost_http_client::models::{UTxO as BFUTxO, Value as BFValue};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    builders::input_builder::{InputBuilderResult, SingleInputBuilder},
    builders::tx_builder::{TransactionBuilder, TransactionBuilderConfigBuilder},
    crypto::TransactionHash,
    ledger::alonzo::fees::LinearFee,
    ledger::common::value::{BigNum, Int, Value as CMLValue},
    plutus::{CostModel, Costmdls, ExUnitPrices, Language},
    AssetName, Assets, MultiAsset, PolicyID, TransactionInput, TransactionOutput, UnitInterval,
};

// TODO: I think some of thise values might be dynamic, in which case we should query them
//   rather than hard-coding them
pub fn test_v1_tx_builder() -> TransactionBuilder {
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

pub(crate) fn input_from_utxo(my_address: &CMLAddress, utxo: &UTxO) -> InputBuilderResult {
    let index = utxo.output_index();
    let tx_hash = utxo.tx_hash();
    let payment_input = TransactionInput::new(
        &tx_hash, // tx hash
        &index,   // index
    );
    let value = utxo.amount();
    let utxo_info = TransactionOutput::new(my_address, &value);
    let input_builder = SingleInputBuilder::new(&payment_input, &utxo_info);

    input_builder.payment_key().unwrap()
}

pub fn cmlvalue_from_bfvalues(values: &[BFValue]) -> CMLValue {
    let mut cml_value = CMLValue::zero();
    for value in values.iter() {
        let unit = value.unit();
        let quantity = value.quantity();
        let add_value = match unit {
            "lovelace" => CMLValue::new(&BigNum::from_str(quantity).unwrap()),
            _ => {
                let policy_id_hex = &unit[..56];
                let policy_id = PolicyID::from_hex(policy_id_hex).unwrap();
                let asset_name_hex = &unit[56..];
                let asset_name_bytes = hex::decode(asset_name_hex).unwrap();
                let asset_name = AssetName::new(asset_name_bytes).unwrap();
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

impl TryFrom<Values> for CMLValue {
    type Error = CMLLCError;

    fn try_from(mut vals: Values) -> Result<Self> {
        if let Some(ada) = vals.take(&PolicyId::ADA) {
            let coin = ada.into();
            let cml_value = CMLValue::new(&coin);
            // let mut multi_asset = MultiAsset::new();
            for (_id, _amount) in vals.as_iter() {
                todo!("Not handling multiasset yet")
                // if let Some(id_str) = id.to_str {
                //     let policy_id = ScriptHash::from_hex(id_str).unwrap(); // TODO: unwrap
                //     let assets = Assets::new();
                //     multi_asset.insert(&policy_id, &assets);
                // }
            }
            // cml_value.set_multiasset(&multi_asset);
            Ok(cml_value)
        } else {
            Err(CMLLCError::InsufficientADA)
        }
    }
}

pub(crate) fn bf_utxo_to_wallet_utxo<Datum>(utxo: &BFUTxO, owner: &Address) -> Output<Datum> {
    let tx_hash = utxo.tx_hash().to_owned();
    let index = utxo.output_index().to_owned();
    let mut values = Values::default();
    utxo.amount()
        .iter()
        .map(as_nau_value)
        .for_each(|(policy_id, amount)| values.add_one_value(&policy_id, amount));
    Output::new_wallet(tx_hash, index, owner.to_owned(), values)
}

pub(crate) fn bf_utxo_to_validator_utxo<Datum>(
    utxo: &BFUTxO,
    owner: &Address,
    datum: Datum,
) -> Output<Datum> {
    let tx_hash = utxo.tx_hash().to_owned();
    let index = utxo.output_index().to_owned();
    let mut values = Values::default();
    utxo.amount()
        .iter()
        .map(as_nau_value)
        .for_each(|(policy_id, amount)| values.add_one_value(&policy_id, amount));
    Output::new_validator(tx_hash, index, owner.to_owned(), values, datum)
}

fn as_nau_value(value: &BFValue) -> (PolicyId, u64) {
    let policy_id = match value.unit() {
        "lovelace" => PolicyId::ADA,
        native_token => {
            let policy = &native_token[..56]; // TODO: Use the rest as asset info
            PolicyId::native_token(policy)
        }
    };
    let amount = value.quantity().parse().unwrap(); // TODO: unwrap
    (policy_id, amount)
}
