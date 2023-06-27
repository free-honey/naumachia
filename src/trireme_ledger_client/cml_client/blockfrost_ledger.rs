use super::error::*;
use crate::trireme_ledger_client::cml_client::{error::CMLLCError, ExecutionCost, Ledger, UTxO};
use async_trait::async_trait;
use blockfrost_http_client::error::Error;
use blockfrost_http_client::{
    models::ExecutionType, models::UTxO as BFUTxO, models::Value as BFValue, BlockFrostHttp,
    BlockFrostHttpTrait,
};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress, crypto::TransactionHash, ledger::common::value::BigNum,
    ledger::common::value::Value as CMLValue, plutus::encode_json_str_to_plutus_datum,
    plutus::PlutusDatumSchema, AssetName, Assets, MultiAsset, PolicyID,
    Transaction as CMLTransaction,
};
use futures::future;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

pub struct BlockFrostLedger {
    client: BlockFrostHttp,
}

impl BlockFrostLedger {
    pub fn new(url: &str, key: &str) -> Self {
        let client = BlockFrostHttp::new(url, key);
        BlockFrostLedger { client }
    }

    // TODO: Handle V2 outputs (with inline datums)
    async fn bfutxo_to_utxo(&self, bf_utxo: &BFUTxO) -> Result<UTxO> {
        let tx_hash = TransactionHash::from_hex(bf_utxo.tx_hash())
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
        let output_index = bf_utxo.output_index().into();
        let amount = cmlvalue_from_bfvalues(bf_utxo.amount())?;
        let datum = if let Some(data_hash) = bf_utxo.data_hash() {
            let json_datum = self
                .client
                .datum(data_hash)
                .await
                .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
            if let Some(inner) = json_datum.as_object() {
                if inner.get("error").is_none() {
                    let ser = json_datum["json_value"].to_string();
                    let plutus_data = encode_json_str_to_plutus_datum(
                        &ser, // TODO: Make this safer!
                        PlutusDatumSchema::DetailedSchema,
                    )
                    .map_err(|e| CMLLCError::JsError(e.to_string()))?;
                    Some(plutus_data)
                } else {
                    None // TODO: Add debug msg
                }
            } else {
                None // TODO: Add debug msg
            }
        } else {
            None
        };

        let utxo = UTxO::new(tx_hash, output_index, amount, datum);
        Ok(utxo)
    }
}

pub fn cmlvalue_from_bfvalues(values: &[BFValue]) -> Result<CMLValue> {
    let mut cml_value = CMLValue::zero();
    for value in values.iter() {
        let unit = value.unit();
        let quantity = value.quantity();
        let add_value = match unit {
            "lovelace" => CMLValue::new(&BigNum::from_str(quantity).unwrap()),
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
                let big_number =
                    BigNum::from_str(quantity).map_err(|e| CMLLCError::JsError(e.to_string()))?;
                assets.insert(&asset_name, &big_number);
                let mut multi_assets = MultiAsset::new();
                multi_assets.insert(&policy_id, &assets);
                CMLValue::new_from_assets(&multi_assets)
            }
        };
        cml_value = cml_value.checked_add(&add_value).unwrap();
    }
    Ok(cml_value)
}

#[async_trait]
impl Ledger for BlockFrostLedger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress, count: usize) -> Result<Vec<UTxO>> {
        let addr_string = addr
            .to_bech32(None)
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
        let bf_utxos_res = self.client.utxos(&addr_string, Some(count)).await;
        let bf_utxos = match bf_utxos_res {
            Ok(bf_utxos) => Ok(bf_utxos),
            Err(e) => match e {
                Error::HttpError { status_code, .. } => {
                    if status_code == 404 {
                        Ok(Vec::new())
                    } else {
                        Err(CMLLCError::LedgerError(Box::new(e)))
                    }
                }
                _ => Err(CMLLCError::LedgerError(Box::new(e))),
            },
        }?;
        let utxos = future::join_all(
            bf_utxos
                .iter()
                .map(|bf_utxo| async move { self.bfutxo_to_utxo(bf_utxo).await }),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        Ok(utxos)
    }

    async fn get_all_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        let addr_string = addr
            .to_bech32(None)
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
        let bf_utxos_res = self.client.utxos(&addr_string, None).await;
        let bf_utxos = match bf_utxos_res {
            Ok(bf_utxos) => Ok(bf_utxos),
            Err(e) => match e {
                Error::HttpError { status_code, .. } => {
                    if status_code == 404 {
                        Ok(Vec::new())
                    } else {
                        Err(CMLLCError::LedgerError(Box::new(e)))
                    }
                }
                _ => Err(CMLLCError::LedgerError(Box::new(e))),
            },
        }?;
        let utxos = future::join_all(
            bf_utxos
                .iter()
                .map(|bf_utxo| async move { self.bfutxo_to_utxo(bf_utxo).await }),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        Ok(utxos)
    }

    async fn calculate_ex_units(&self, tx: &CMLTransaction) -> Result<HashMap<u64, ExecutionCost>> {
        let bytes = tx.to_bytes();
        let res = self
            .client
            .execution_units(&bytes)
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
        let bf_spends = res
            .get_execution_costs()
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
        let spends = bf_spends
            .iter()
            .map(|(index, bf_spend)| (*index, spend_from_bf_spend(bf_spend)))
            .collect();
        // dbg!(spends);
        // todo!();
        Ok(spends)
    }

    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String> {
        println!("{}", &tx.to_json().unwrap());
        let res = self
            .client
            .submit_tx(&tx.to_bytes())
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
        Ok(res.tx_id().to_string())
    }
}

fn spend_from_bf_spend(
    bf_spend: &blockfrost_http_client::models::ExecutionCostsWithType,
) -> ExecutionCost {
    let memory = bf_spend.memory();
    let steps = bf_spend.steps();
    match bf_spend.execution_type() {
        ExecutionType::Spend => ExecutionCost::new_spend(memory, steps),
        ExecutionType::Mint => ExecutionCost::new_mint(memory, steps),
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlockfrostApiKey {
    inner: String,
}

impl From<BlockfrostApiKey> for String {
    fn from(secret_phrase: BlockfrostApiKey) -> Self {
        secret_phrase.inner
    }
}

impl From<&BlockfrostApiKey> for String {
    fn from(secret_phrase: &BlockfrostApiKey) -> Self {
        secret_phrase.inner.clone()
    }
}

impl FromStr for BlockfrostApiKey {
    type Err = BlockfrostLedgerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.to_string();
        Ok(BlockfrostApiKey { inner })
    }
}

#[derive(Debug, Error)]
pub enum BlockfrostLedgerError {
    #[error("No config directory for raw phrase file: {0:?}")]
    NoConfigDirectory(String),
}
