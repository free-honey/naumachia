use super::error::*;
use crate::{
    ledger_client::cml_client::error::CMLLCError::JsError, ledger_client::cml_client::Ledger,
};
use async_trait::async_trait;
use blockfrost_http_client::models::UTxO as BFUTxO;
use blockfrost_http_client::{BlockFrostHttp, BlockFrostHttpTrait};
use cardano_multiplatform_lib::address::Address as CMLAddress;
use cardano_multiplatform_lib::Transaction as CMLTransaction;

pub struct BlockFrostLedger {
    client: BlockFrostHttp,
}

impl BlockFrostLedger {
    pub fn new(url: &str, key: &str) -> Self {
        let client = BlockFrostHttp::new(url, key);
        BlockFrostLedger { client }
    }
}

#[async_trait]
impl Ledger for BlockFrostLedger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<BFUTxO>> {
        let addr_string = addr.to_bech32(None).map_err(|e| JsError(e.to_string()))?;
        let utxos = self
            .client
            .utxos(&addr_string)
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
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
