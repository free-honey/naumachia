use super::error::*;
use crate::scripts::MintingPolicy;
use crate::{
    ledger_client::{LedgerClientError, LedgerClientResult},
    output::Output,
    scripts::ValidatorCode,
    trireme_ledger_client::cml_client::{
        error::CMLLCError::JsError, plutus_data_interop::PlutusDataInterop, UTxO,
    },
    values::Values,
    Address, PolicyId,
};
use blockfrost_http_client::models::Value as BFValue;
use cardano_multiplatform_lib::crypto::ScriptHash;
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    builders::{
        input_builder::{InputBuilderResult, SingleInputBuilder},
        output_builder::TransactionOutputBuilder,
        tx_builder::{ChangeSelectionAlgo, CoinSelectionStrategyCIP2, SignedTxBuilder},
        tx_builder::{TransactionBuilder, TransactionBuilderConfigBuilder},
        witness_builder::{PartialPlutusWitness, PlutusScriptWitness},
    },
    crypto::{PrivateKey, TransactionHash},
    ledger::{
        alonzo::fees::LinearFee,
        common::hash::hash_transaction,
        common::value::{BigNum, Int, Value as CMLValue},
        shelley::witness::make_vkey_witness,
    },
    plutus::{CostModel, Costmdls, ExUnitPrices, Language},
    plutus::{PlutusScript, PlutusV1Script},
    AssetName, Assets, MultiAsset, PolicyID, Transaction as CMLTransaction, TransactionInput,
    TransactionOutput, UnitInterval,
};
use std::collections::BTreeMap;

// TODO: I think some of these values might be dynamic, in which case we should query them
//   rather than hard-coding them
pub fn vasil_v1_tx_builder() -> LedgerClientResult<TransactionBuilder> {
    let coefficient = 44.into();
    let constant = 155381.into();
    let linear_fee = LinearFee::new(&coefficient, &constant);

    let pool_deposit = 500000000.into();
    let key_deposit = 2000000.into();

    let coins_per_utxo_byte = 4310.into();
    let mem_num = 577.into();
    let mem_den = 10000.into();
    let mem_price = UnitInterval::new(&mem_num, &mem_den);
    let step_num = 721.into();
    let step_den = 10000000.into();
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
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(TransactionBuilder::new(&tx_builder_cfg))
}

// TODO: I think some of these values might be dynamic, in which case we should query them
//   rather than hard-coding them
pub fn vasil_v2_tx_builder() -> LedgerClientResult<TransactionBuilder> {
    let coefficient = 44.into();
    let constant = 155381.into();
    let linear_fee = LinearFee::new(&coefficient, &constant);

    let pool_deposit = 500000000.into();
    let key_deposit = 2000000.into();

    let coins_per_utxo_byte = 4310.into();
    let mem_num = 577.into();
    let mem_den = 10000.into();
    let mem_price = UnitInterval::new(&mem_num, &mem_den);
    let step_num = 721.into();
    let step_den = 10000000.into();
    let step_price = UnitInterval::new(&step_num, &step_den);
    let ex_unit_prices = ExUnitPrices::new(&mem_price, &step_price);
    let vasil_v2_cost_models = vec![
        205665,
        812,
        1,
        1,
        1000,
        571,
        0,
        1,
        1000,
        24177,
        4,
        1,
        1000,
        32,
        117366,
        10475,
        4,
        23000,
        100,
        23000,
        100,
        23000,
        100,
        23000,
        100,
        23000,
        100,
        23000,
        100,
        100,
        100,
        23000,
        100,
        19537,
        32,
        175354,
        32,
        46417,
        4,
        221973,
        511,
        0,
        1,
        89141,
        32,
        497525,
        14068,
        4,
        2,
        196500,
        453240,
        220,
        0,
        1,
        1,
        1000,
        28662,
        4,
        2,
        245000,
        216773,
        62,
        1,
        1060367,
        12586,
        1,
        208512,
        421,
        1,
        187000,
        1000,
        52998,
        1,
        80436,
        32,
        43249,
        32,
        1000,
        32,
        80556,
        1,
        57667,
        4,
        1000,
        10,
        197145,
        156,
        1,
        197145,
        156,
        1,
        204924,
        473,
        1,
        208896,
        511,
        1,
        52467,
        32,
        64832,
        32,
        65493,
        32,
        22558,
        32,
        16563,
        32,
        76511,
        32,
        196500,
        453240,
        220,
        0,
        1,
        1,
        69522,
        11687,
        0,
        1,
        60091,
        32,
        196500,
        453240,
        220,
        0,
        1,
        1,
        196500,
        453240,
        220,
        0,
        1,
        1,
        1159724,
        392670,
        0,
        2,
        806990,
        30482,
        4,
        1927926,
        82523,
        4,
        265318,
        0,
        4,
        0,
        85931,
        32,
        205665,
        812,
        1,
        1,
        41182,
        32,
        212342,
        32,
        31220,
        32,
        32696,
        32,
        43357,
        32,
        32247,
        32,
        38314,
        32,
        20000000000,
        20000000000,
        9462713,
        1021,
        10,
        20000000000,
        0,
        20000000000,
    ];
    let cm = CostModel::new(
        &Language::new_plutus_v2(),
        &vasil_v2_cost_models.iter().map(|&i| Int::from(i)).collect(),
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
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(TransactionBuilder::new(&tx_builder_cfg))
}

pub(crate) fn input_from_utxo(
    my_address: &CMLAddress,
    utxo: &UTxO,
) -> LedgerClientResult<InputBuilderResult> {
    let index = utxo.output_index();
    let tx_hash = utxo.tx_hash();
    let payment_input = TransactionInput::new(
        tx_hash, // tx hash
        &index,  // index
    );
    let value = utxo.amount();
    let utxo_info = TransactionOutput::new(my_address, value);
    let input_builder = SingleInputBuilder::new(&payment_input, &utxo_info);

    let res = input_builder
        .payment_key()
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(res)
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

    fn try_from(vals: Values) -> Result<Self> {
        let mut ada = 1120600u64; // TODO: This is kinda buried. Maybe ref the CML value
        let mut nau_assets: BTreeMap<String, BTreeMap<Option<String>, u64>> = BTreeMap::new();
        for (policy_id, amount) in vals.as_iter() {
            match policy_id {
                PolicyId::ADA => ada = *amount,
                PolicyId::NativeToken(id, asset_name) => {
                    if let Some(mut inner) = nau_assets.remove(id) {
                        inner.insert(asset_name.to_owned(), *amount);
                        nau_assets.insert(id.to_owned(), inner);
                    } else {
                        let mut inner = BTreeMap::new();
                        inner.insert(asset_name.to_owned(), *amount);
                        nau_assets.insert(id.to_owned(), inner);
                    }
                }
            }
        }
        let coin = ada.into();
        let mut cml_value = CMLValue::new(&coin);
        let mut multi_asset = MultiAsset::new();
        for (id, assets) in nau_assets.iter() {
            let mut cml_assets = Assets::new();
            for (name, amount) in assets.iter() {
                let key = if let Some(inner) = name {
                    AssetName::new(inner.as_bytes().to_owned())
                } else {
                    AssetName::new(Vec::new())
                }
                .unwrap(); // TODO
                let value = (*amount).into();
                cml_assets.insert(&key, &value);
            }
            let policy_id = ScriptHash::from_hex(id).unwrap(); // TODO
            multi_asset.insert(&policy_id, &cml_assets);
        }
        cml_value.set_multiasset(&multi_asset);
        Ok(cml_value)
    }
}

pub(crate) fn utxo_to_nau_utxo<Datum: PlutusDataInterop>(
    utxo: &UTxO,
    owner: &Address,
) -> LedgerClientResult<Output<Datum>> {
    let tx_hash = utxo.tx_hash().to_string();
    let index = utxo.output_index().into();
    let values = as_nau_values(utxo.amount())?;

    // TODO: Add debug msg in the case that this can't convert from PlutusData?
    let output = if let Some(datum) = utxo
        .datum()
        .to_owned()
        .and_then(|data| Datum::from_plutus_data(&data).ok())
    {
        Output::new_validator(tx_hash, index, owner.to_owned(), values, datum)
    } else {
        Output::new_wallet(tx_hash, index, owner.to_owned(), values)
    };
    Ok(output)
}

fn as_nau_values(cml_value: &CMLValue) -> LedgerClientResult<Values> {
    let mut values = Values::default();
    let ada = cml_value.coin().into();
    values.add_one_value(&PolicyId::ADA, ada);
    if let Some(multiasset) = cml_value.multiasset() {
        let ids = multiasset.keys();
        let len = multiasset.len();
        for i in 0..len {
            let id = ids.get(i);
            if let Some(assets) = multiasset.get(&id) {
                let assets_names = assets.keys();
                let len = assets_names.len();
                for i in 0..len {
                    let asset = assets_names.get(i);
                    if let Some(amt) = assets.get(&asset) {
                        let asset_bytes = asset.to_bytes();
                        let asset_text =
                            std::str::from_utf8(&asset_bytes).map_err(as_failed_to_issue_tx)?;
                        let policy_id = PolicyId::native_token(
                            &id.to_string(),
                            &Some(asset_text.to_string()), // TODO: What if there is no assetname?
                        );
                        values.add_one_value(&policy_id, amt.into());
                    }
                }
            }
        }
    }

    Ok(values)
}

pub(crate) async fn specify_utxos_available_for_selection(
    tx_builder: &mut TransactionBuilder,
    my_address: &CMLAddress,
    my_utxos: &[UTxO],
) -> LedgerClientResult<()> {
    for utxo in my_utxos.iter() {
        let input = input_from_utxo(my_address, utxo)?;
        tx_builder.add_utxo(&input);
    }
    Ok(())
}

pub(crate) async fn add_collateral(
    tx_builder: &mut TransactionBuilder,
    my_address: &CMLAddress,
    my_utxos: &Vec<UTxO>,
) -> LedgerClientResult<()> {
    const MIN_COLLATERAL_AMT: u64 = 5_000_000;

    let collateral_utxo = select_collateral_utxo(my_address, my_utxos, MIN_COLLATERAL_AMT)?;

    tx_builder
        .add_collateral(&collateral_utxo)
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(())
}

pub(crate) async fn select_inputs_from_utxos(
    tx_builder: &mut TransactionBuilder,
) -> LedgerClientResult<()> {
    // Hardcode for now. I'm choosing this strat because it helps atomize my wallet a
    // little more which makes testing a bit safer 🦺
    let strategy = CoinSelectionStrategyCIP2::LargestFirstMultiAsset;
    tx_builder
        .select_utxos(strategy)
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(())
}

// TODO: This could be less naive (e.g. include multiple UTxOs, etc)
pub(crate) fn select_collateral_utxo(
    my_cml_address: &CMLAddress,
    my_utxos: &Vec<UTxO>,
    min_amount: u64,
) -> LedgerClientResult<InputBuilderResult> {
    let mut smallest_utxo_meets_qual = None;
    let mut smallest_amount = min_amount;
    for utxo in my_utxos {
        if utxo.amount().multiasset().is_none() {
            let amount: u64 = utxo.amount().coin().into();
            if amount < smallest_amount {
                smallest_utxo_meets_qual = Some(utxo);
                smallest_amount = amount;
            }
        }
    }
    let res = if let Some(utxo) = smallest_utxo_meets_qual {
        let transaction_input = TransactionInput::new(utxo.tx_hash(), &utxo.output_index());
        let input_utxo = TransactionOutputBuilder::new()
            .with_address(my_cml_address)
            .next()
            .map_err(|e| JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?
            .with_coin(&smallest_amount.into())
            .build()
            .map_err(|e| JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?
            .output();
        let res = SingleInputBuilder::new(&transaction_input, &input_utxo)
            .payment_key()
            .map_err(|e| JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Some(res)
    } else {
        None
    };
    res.ok_or(LedgerClientError::NoBigEnoughCollateralUTxO)
}

pub(crate) async fn build_tx_for_signing(
    tx_builder: &mut TransactionBuilder,
    my_address: &CMLAddress,
) -> LedgerClientResult<SignedTxBuilder> {
    let algo = ChangeSelectionAlgo::Default;
    let signed_tx_builder = tx_builder
        .build(algo, my_address)
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(signed_tx_builder)
}

pub(crate) async fn sign_tx(
    signed_tx_builder: &mut SignedTxBuilder,
    priv_key: &PrivateKey,
) -> LedgerClientResult<CMLTransaction> {
    let unchecked_tx = signed_tx_builder.build_unchecked();
    let tx_body = unchecked_tx.body();
    let tx_hash = hash_transaction(&tx_body);
    let vkey_witness = make_vkey_witness(&tx_hash, priv_key);
    signed_tx_builder.add_vkey(&vkey_witness);
    let tx = signed_tx_builder
        .build_checked()
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(tx)
}

pub(crate) async fn input_tx_hash<Datum>(
    input: &Output<Datum>,
) -> LedgerClientResult<TransactionHash> {
    let tx_hash_raw = input.id().tx_hash();
    let tx_hash = TransactionHash::from_hex(tx_hash_raw)
        .map_err(|e| CMLLCError::JsError(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    Ok(tx_hash)
}

pub(crate) async fn cml_v1_script_from_nau_script<Datum, Redeemer>(
    script: &(dyn ValidatorCode<Datum, Redeemer> + '_),
) -> LedgerClientResult<PlutusScript> {
    let script_hex = script.script_hex().map_err(as_failed_to_issue_tx)?;
    let script_bytes = hex::decode(script_hex).map_err(as_failed_to_issue_tx)?;
    let v1 = PlutusV1Script::from_bytes(script_bytes)
        .map_err(|e| CMLLCError::Deserialize(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    let cml_script = PlutusScript::from_v1(&v1);
    Ok(cml_script)
}

pub(crate) async fn cml_v1_script_from_nau_policy<Redeemer>(
    script: &(dyn MintingPolicy<Redeemer> + '_),
) -> LedgerClientResult<PlutusScript> {
    let script_hex = script.script_hex().map_err(as_failed_to_issue_tx)?;
    let script_bytes = hex::decode(script_hex).map_err(as_failed_to_issue_tx)?;
    let v1 = PlutusV1Script::from_bytes(script_bytes)
        .map_err(|e| CMLLCError::Deserialize(e.to_string()))
        .map_err(as_failed_to_issue_tx)?;
    let cml_script = PlutusScript::from_v1(&v1);
    Ok(cml_script)
}

pub(crate) async fn partial_script_witness<Redeemer: PlutusDataInterop>(
    cml_script: &PlutusScript,
    redeemer: &Redeemer,
) -> PartialPlutusWitness {
    let script_witness = PlutusScriptWitness::from_script(cml_script.clone());
    PartialPlutusWitness::new(&script_witness, &redeemer.to_plutus_data())
}
