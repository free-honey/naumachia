use crate::trireme_ledger_client::cml_client::network_settings::NetworkSettings;
use crate::trireme_ledger_client::cml_client::{
    error::{CMLLCError, Result},
    ExecutionCost, Ledger, UTxO,
};
use async_trait::async_trait;
use cardano_multiplatform_lib::{
    address::Address as CMLAddress, crypto::TransactionHash,
    ledger::common::value::Value as CMLValue, plutus::PlutusData, AssetName, Assets, MultiAsset,
    PolicyID, Transaction as CMLTransaction,
};
use ogmios_client::{EvaluationResult, OgmiosClient, OgmiosLocalTxSubmission, OgmiosResponse};
use pallas_addresses::Address;
use scrolls_client::{
    Amount as ScrollClientAmount, LastBlockInfo, ScrollsClient, UTxO as ScrollsClientUTxO,
    UTxOsByAddress,
};
use std::collections::HashMap;

fn utxo_from_scrolls_utxo(utxo: &ScrollsClientUTxO) -> Result<UTxO> {
    let tx_hash = TransactionHash::from_hex(utxo.tx_hash())
        .map_err(|e| CMLLCError::JsError(e.to_string()))?;
    let output_index = utxo.output_index().into();
    let scroll_amount = utxo.amount();
    let amount = cml_value_from_scroll_amount(scroll_amount)?;
    let maybe_datum = utxo.datum();
    let datum = if let Some(datum) = maybe_datum {
        plutus_data_from_scroll_datum(datum)?
    } else {
        None
    };

    Ok(UTxO::new(tx_hash, output_index, amount, datum))
}

fn cml_value_from_scroll_amount(amount: &[ScrollClientAmount]) -> Result<CMLValue> {
    let mut cml_value = CMLValue::zero();
    for value in amount.iter() {
        let unit = value.unit();
        let quantity = value.quantity();
        let add_value = match unit {
            "lovelace" => CMLValue::new(&quantity.into()),
            _ => {
                let policy_id_hex = &unit
                    .get(..56)
                    .ok_or(CMLLCError::InvalidPolicyId(unit.to_string()))?;
                let policy_id = PolicyID::from_hex(policy_id_hex)
                    .map_err(|e| CMLLCError::JsError(e.to_string()))?;
                let asset_name_hex = &unit
                    .get(56..)
                    .ok_or(CMLLCError::InvalidPolicyId(unit.to_string()))?;
                let asset_name_bytes =
                    hex::decode(asset_name_hex).map_err(|e| CMLLCError::JsError(e.to_string()))?;
                let asset_name = AssetName::new(asset_name_bytes)
                    .map_err(|e| CMLLCError::JsError(e.to_string()))?;
                let mut assets = Assets::new();
                assets.insert(&asset_name, &quantity.into());
                let mut multi_assets = MultiAsset::new();
                multi_assets.insert(&policy_id, &assets);
                CMLValue::new_from_assets(&multi_assets)
            }
        };
        cml_value = cml_value
            .checked_add(&add_value)
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
    }
    Ok(cml_value)
}

fn plutus_data_from_scroll_datum(datum: &str) -> Result<Option<PlutusData>> {
    let bytes = hex::decode(datum)?;
    Ok(PlutusData::from_bytes(bytes).ok())
}

pub struct OgmiosScrollsLedger {
    scrolls_client: ScrollsClient,
    ogmios_client: OgmiosClient,
    network_settings: NetworkSettings,
    // TODO: WS Client for Ogmios data
}

impl OgmiosScrollsLedger {
    pub fn new(
        scrolls_client: ScrollsClient,
        ogmios_client: OgmiosClient,
        network_settings: NetworkSettings,
    ) -> Self {
        Self {
            scrolls_client,
            ogmios_client,
            network_settings,
        }
    }

    pub async fn get_utxos(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        let address_str = addr
            .to_bech32(None)
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
        let address = Address::from_bech32(&address_str)?;
        self.scrolls_client
            .get_utxos_for_address(&address)
            .await?
            .iter()
            .map(utxo_from_scrolls_utxo)
            .collect()
    }
}

#[async_trait]
impl Ledger for OgmiosScrollsLedger {
    async fn last_block_time_secs(&self) -> Result<i64> {
        let slot = self.scrolls_client.get_last_block_info().await?.slot;
        Ok(self.network_settings.posix_from_slot(slot))
    }

    async fn get_utxos_for_addr(&self, addr: &CMLAddress, count: usize) -> Result<Vec<UTxO>> {
        let outputs = self
            .get_utxos(addr)
            .await?
            .into_iter()
            .take(count)
            .collect();
        Ok(outputs)
    }

    async fn get_all_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        self.get_utxos(addr).await
    }

    async fn calculate_ex_units(&self, tx: &CMLTransaction) -> Result<HashMap<u64, ExecutionCost>> {
        let bytes = tx.to_bytes();
        let res = self.ogmios_client.evaluate_tx(&bytes, vec![]).await?;
        check_for_error(&res)?;
        parse_evaluation_results(&res)
    }

    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String> {
        let bytes = tx.to_bytes();
        let res = self.ogmios_client.submit_tx(&bytes).await?;
        let tx_hash = res
            .result()
            .ok_or(CMLLCError::OgmiosResponse(
                "No transaction hash in response".to_string(),
            ))?
            .tx_id()
            .to_string();
        Ok(tx_hash)
    }
}

fn check_for_error(res: &OgmiosResponse<EvaluationResult>) -> Result<()> {
    if let Some(err) = res.fault() {
        Err(CMLLCError::OgmiosResponse(err.to_string()))
    } else {
        Ok(())
    }
}

fn parse_evaluation_results(
    res: &OgmiosResponse<EvaluationResult>,
) -> Result<HashMap<u64, ExecutionCost>> {
    let eval_res = if let Some(eval_res) = res.result() {
        Ok(eval_res)
    } else {
        Err(CMLLCError::OgmiosResponse(
            "No evaluation result".to_string(),
        ))
    }?;
    let json = eval_res.value();
    let map = json
        .as_object()
        .and_then(|inner| inner.get("EvaluationResult"))
        .and_then(|inner| inner.as_object())
        .ok_or(CMLLCError::OgmiosResponse(
            "Evaluation result is not a JSON object".to_string(),
        ))?
        .iter()
        .filter_map(|(k, v)| parse_pair(k, v))
        .collect();
    Ok(map)
}

fn parse_pair(key: &str, value: &serde_json::Value) -> Option<(u64, ExecutionCost)> {
    let split_key = key.split(':').collect::<Vec<_>>();
    let index = split_key.get(1).and_then(|s| s.parse::<u64>().ok())?;
    let type_str = split_key.first()?;
    let value_obj = value.as_object()?;
    let memory = value_obj.get("memory")?.as_u64()?;
    let steps = value_obj.get("steps")?.as_u64()?;
    let ex_cost = match *type_str {
        "spend" => Some(ExecutionCost::new_spend(memory, steps)),
        "mint" => Some(ExecutionCost::new_mint(memory, steps)),
        "withdrawal" => Some(ExecutionCost::new_withdrawal(memory, steps)),
        "certificate" => Some(ExecutionCost::new_certificate(memory, steps)),
        _ => None,
    }?;
    Some((index, ex_cost))
}
