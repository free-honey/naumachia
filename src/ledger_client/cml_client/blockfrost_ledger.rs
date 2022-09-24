use super::error::*;
use crate::ledger_client::cml_client::issuance_helpers::cmlvalue_from_bfvalues;
use crate::ledger_client::cml_client::{cmlvalues_from_values, UTxO};
use crate::{
    ledger_client::cml_client::error::CMLLCError::JsError, ledger_client::cml_client::Ledger,
};
use async_trait::async_trait;
use blockfrost_http_client::models::UTxO as BFUTxO;
use blockfrost_http_client::{BlockFrostHttp, BlockFrostHttpTrait};
use cardano_multiplatform_lib::address::Address as CMLAddress;
use cardano_multiplatform_lib::crypto::TransactionHash;
use cardano_multiplatform_lib::error::JsError;
use cardano_multiplatform_lib::plutus::{
    encode_json_value_to_plutus_datum, PlutusData, PlutusDatumSchema,
};
use cardano_multiplatform_lib::Transaction as CMLTransaction;
use futures::future;

pub struct BlockFrostLedger {
    client: BlockFrostHttp,
}

impl BlockFrostLedger {
    pub fn new(url: &str, key: &str) -> Self {
        let client = BlockFrostHttp::new(url, key);
        BlockFrostLedger { client }
    }

    async fn bfutxo_to_utxo(&self, bf_utxo: &BFUTxO) -> Result<UTxO> {
        let tx_hash =
            TransactionHash::from_hex(bf_utxo.tx_hash()).map_err(|e| JsError(e.to_string()))?;
        let output_index = bf_utxo.output_index().into();
        let amount = cmlvalue_from_bfvalues(bf_utxo.amount());
        let block = bf_utxo.block().to_string();
        let datum = if let Some(data_hash) = bf_utxo.data_hash() {
            let json_datum = self
                .client
                .datum(data_hash)
                .await
                .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
            if let Some(inner) = json_datum.as_object() {
                if inner.get("error").is_none() {
                    let plutus_data = encode_json_value_to_plutus_datum(
                        json_datum["json_value"].clone(), // TODO: Make this safer!
                        PlutusDatumSchema::DetailedSchema,
                    )
                    .map_err(|e| JsError(e.to_string()));
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

        let utxo = UTxO::new(tx_hash, output_index, amount, block, datum);
        Ok(utxo)
    }
}

#[async_trait]
impl Ledger for BlockFrostLedger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        let addr_string = addr.to_bech32(None).map_err(|e| JsError(e.to_string()))?;
        let bf_utxos = self
            .client
            .utxos(&addr_string)
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
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

    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String> {
        let res = self
            .client
            .submit_tx(&tx.to_bytes())
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
        Ok(res.tx_id().to_string())
    }
}
