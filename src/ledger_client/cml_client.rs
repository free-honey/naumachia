use crate::{
    ledger_client::{
        cml_client::issuance_helpers::{bf_utxo_to_utxo, input_from_utxo, test_v1_tx_builder},
        LedgerClient, LedgerClientResult,
    },
    output::{Output, UnbuiltOutput},
    transaction::TxId,
    Address, UnbuiltTransaction,
};
use async_trait::async_trait;
use blockfrost_http_client::models::UTxO as BFUTxO;
use cardano_multiplatform_lib::builders::tx_builder::TransactionBuilder;
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    builders::{
        output_builder::SingleOutputBuilderResult,
        tx_builder::{ChangeSelectionAlgo, CoinSelectionStrategyCIP2},
    },
    crypto::PrivateKey,
    ledger::{
        common::hash::hash_transaction, common::value::Value as CMLValue,
        shelley::witness::make_vkey_witness,
    },
    Transaction as CMLTransaction, TransactionOutput,
};
use error::*;
use std::marker::PhantomData;

pub mod blockfrost_ledger;
mod error;
mod issuance_helpers;
pub mod key_manager;

#[cfg(test)]
mod tests;

// TODO: Add minimum ADA https://github.com/MitchTurner/naumachia/issues/41
pub struct CMLLedgerCLient<L, K, Datum, Redeemer>
where
    L: Ledger,
    K: Keys,
{
    ledger: L,
    keys: K,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

#[async_trait]
pub trait Keys {
    async fn base_addr(&self) -> Result<CMLAddress>;
    async fn private_key(&self) -> Result<PrivateKey>;
    async fn addr_from_bech_32(&self, addr: &str) -> Result<CMLAddress>;
}

#[async_trait]
pub trait Ledger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<BFUTxO>>; // TODO: Don't take dep on BF UTxO
    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String>;
}

impl<L, K, D, R> CMLLedgerCLient<L, K, D, R>
where
    L: Ledger,
    K: Keys,
{
    pub fn new(ledger: L, keys: K) -> Self {
        CMLLedgerCLient {
            ledger,
            keys,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }

    async fn add_outputs_for_tx<Datum, Redeemer>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for unbuilt_output in tx.unbuilt_outputs().iter() {
            let cml_values: CMLValue = unbuilt_output
                .values()
                .to_owned()
                .try_into()
                .map_err(as_failed_to_issue_tx)?;
            let recipient = unbuilt_output.owner();
            let recp_addr = self
                .keys
                .addr_from_bech_32(recipient.to_str())
                .await
                .map_err(as_failed_to_issue_tx)?;
            let output = TransactionOutput::new(&recp_addr, &cml_values);
            let res = SingleOutputBuilderResult::new(&output);
            if let UnbuiltOutput::Validator { .. } = unbuilt_output {
                todo!("Can't build outputs with datums and stuff yet");
            }
            tx_builder
                .add_output(&res)
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
        }
        Ok(())
    }
}

#[async_trait]
impl<L, K, Datum, Redeemer> LedgerClient<Datum, Redeemer> for CMLLedgerCLient<L, K, Datum, Redeemer>
where
    L: Ledger + Send + Sync,
    K: Keys + Send + Sync,
    Datum: Send + Sync,
    Redeemer: Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<&Address> {
        todo!()
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match address {
            Address::Base(addr_string) => {
                let base_addr = self
                    .keys
                    .addr_from_bech_32(addr_string)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let bf_utxos = self
                    .ledger
                    .get_utxos_for_addr(&base_addr)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let utxos = bf_utxos
                    .iter()
                    .map(|utxo| bf_utxo_to_utxo(utxo, address))
                    .collect();
                Ok(utxos)
            }
            Address::Raw(_) => unimplemented!("Doesn't make sense here"),
        }
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        let my_address = self.keys.base_addr().await.map_err(as_failed_to_issue_tx)?;
        let priv_key = self
            .keys
            .private_key()
            .await
            .map_err(as_failed_to_issue_tx)?;

        let my_utxos = self
            .ledger
            .get_utxos_for_addr(&my_address)
            .await
            .map_err(as_failed_to_issue_tx)?;

        let mut tx_builder = test_v1_tx_builder();

        for utxo in my_utxos.iter() {
            let input = input_from_utxo(&my_address, utxo);
            tx_builder.add_utxo(&input);
        }

        self.add_outputs_for_tx(&mut tx_builder, &tx).await?;

        // Hardcode for now. I'm choosing this strat because it helps atomize my wallet a
        // little more which makes testing a bit safer 🦺
        let strategy = CoinSelectionStrategyCIP2::LargestFirstMultiAsset;
        tx_builder
            .select_utxos(strategy)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let algo = ChangeSelectionAlgo::Default;
        let mut signed_tx_builder = tx_builder
            .build(algo, &my_address)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let unchecked_tx = signed_tx_builder.build_unchecked();
        let tx_body = unchecked_tx.body();
        let tx_hash = hash_transaction(&tx_body);
        let vkey_witness = make_vkey_witness(&tx_hash, &priv_key);
        signed_tx_builder.add_vkey(&vkey_witness);
        let tx = signed_tx_builder
            .build_checked()
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let submit_res = self
            .ledger
            .submit_transaction(&tx)
            .await
            .map_err(as_failed_to_issue_tx)?;
        Ok(TxId::new(&submit_res))
    }
}
