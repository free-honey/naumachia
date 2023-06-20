use crate::trireme_ledger_client::cml_client::{
    error::{CMLLCError, Result},
    ExecutionCost, Ledger, UTxO,
};
use async_trait::async_trait;
use cardano_multiplatform_lib::{address::Address as CMLAddress, Transaction as CMLTransaction};
use pallas_addresses::Address;
use scrolls_client::{ScrollsClient, UTxO as ScrollsClientUTxO, UTxOsByAddress};
use std::collections::HashMap;

impl From<ScrollsClientUTxO> for UTxO {
    fn from(value: ScrollsClientUTxO) -> Self {
        todo!()
    }
}

pub struct OgmiosScrollsLedger {
    scrolls_client: ScrollsClient,
    // TODO: WS Client for Ogmios data
}

impl OgmiosScrollsLedger {
    pub async fn get_utxos(&self, addr: &CMLAddress) -> Result<Vec<UTxO>> {
        let address_str = addr
            .to_bech32(None)
            .map_err(|e| CMLLCError::JsError(e.to_string()))?;
        let address = Address::from_bech32(&address_str)?;
        let outputs = self
            .scrolls_client
            .get_utxos_for_address(&address)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        Ok(outputs)
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
