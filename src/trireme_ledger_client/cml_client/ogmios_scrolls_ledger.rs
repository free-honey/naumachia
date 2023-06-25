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
use pallas_addresses::Address;
use scrolls_client::{
    Amount as ScrollClientAmount, ScrollsClient, UTxO as ScrollsClientUTxO, UTxOsByAddress,
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
    // TODO: WS Client for Ogmios data
}

impl OgmiosScrollsLedger {
    pub fn new(scrolls_client: ScrollsClient) -> Self {
        Self { scrolls_client }
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

    async fn calculate_ex_units(
        &self,
        _tx: &CMLTransaction,
    ) -> Result<HashMap<u64, ExecutionCost>> {
        todo!()
    }

    async fn submit_transaction(&self, _tx: &CMLTransaction) -> Result<String> {
        todo!()
    }
}
