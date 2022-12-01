use super::error::*;
use crate::trireme_ledger_client::cml_client::{
    error::CMLLCError::JsError, issuance_helpers::cmlvalue_from_bfvalues, ExecutionCost, Ledger,
    UTxO,
};
use async_trait::async_trait;
use blockfrost_http_client::models::ExecutionType;
use blockfrost_http_client::{models::UTxO as BFUTxO, BlockFrostHttp, BlockFrostHttpTrait};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    crypto::TransactionHash,
    plutus::{encode_json_value_to_plutus_datum, PlutusDatumSchema},
    Transaction as CMLTransaction,
};
use futures::future;
use std::collections::HashMap;

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
        let tx_hash =
            TransactionHash::from_hex(bf_utxo.tx_hash()).map_err(|e| JsError(e.to_string()))?;
        let output_index = bf_utxo.output_index().into();
        let amount = cmlvalue_from_bfvalues(bf_utxo.amount());
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
                    .map_err(|e| JsError(e.to_string()))?;
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

#[async_trait]
impl Ledger for BlockFrostLedger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress, count: usize) -> Result<Vec<UTxO>> {
        let addr_string = addr.to_bech32(None).map_err(|e| JsError(e.to_string()))?;
        let bf_utxos = self
            .client
            .utxos(&addr_string, Some(count))
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

    async fn get_all_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        let addr_string = addr.to_bech32(None).map_err(|e| JsError(e.to_string()))?;
        let bf_utxos = self
            .client
            .utxos(&addr_string, None)
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
        Ok(spends)
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
