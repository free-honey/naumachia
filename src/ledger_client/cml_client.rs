use crate::ledger_client::cml_client::issuance_helpers::bf_utxo_to_validator_utxo;
use crate::ledger_client::cml_client::plutus_data_interop::PlutusDataInterop;
use crate::values::Values;
use crate::{
    ledger_client::{
        cml_client::issuance_helpers::{
            bf_utxo_to_wallet_utxo, input_from_utxo, test_v1_tx_builder,
        },
        LedgerClient, LedgerClientResult,
    },
    output::{Output, UnbuiltOutput},
    transaction::TxId,
    Address, UnbuiltTransaction,
};
use async_trait::async_trait;
use blockfrost_http_client::models::UTxO as BFUTxO;
use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::builders::input_builder::SingleInputBuilder;
use cardano_multiplatform_lib::builders::tx_builder::TransactionBuilder;
use cardano_multiplatform_lib::builders::witness_builder::{
    PartialPlutusWitness, PlutusScriptWitness,
};
use cardano_multiplatform_lib::crypto::{DataHash, TransactionHash};
use cardano_multiplatform_lib::ledger::common::hash::hash_plutus_data;
use cardano_multiplatform_lib::ledger::common::value::{BigInt, BigNum};
use cardano_multiplatform_lib::plutus::{PlutusData, PlutusScript, PlutusV1Script};
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
    Datum as CMLDatum, RequiredSigners, Transaction as CMLTransaction, TransactionInput,
    TransactionOutput,
};
use error::*;
use std::marker::PhantomData;

pub mod blockfrost_ledger;
mod error;
mod issuance_helpers;
pub mod key_manager;
pub mod plutus_data_interop;

#[cfg(test)]
mod tests;

// TODO: Add minimum ADA https://github.com/MitchTurner/naumachia/issues/41
pub struct CMLLedgerCLient<L, K, Datum, Redeemer>
where
    L: Ledger,
    K: Keys,
    Datum: PlutusDataInterop,
    Redeemer: PlutusDataInterop,
{
    ledger: L,
    keys: K,
    network: u8,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

#[async_trait]
pub trait Keys {
    async fn base_addr(&self) -> Result<CMLAddress>;
    async fn private_key(&self) -> Result<PrivateKey>;
    async fn addr_from_bech_32(&self, addr: &str) -> Result<CMLAddress>;
}

pub struct UTxO {
    tx_hash: TransactionHash,
    output_index: BigNum,
    amount: CMLValue,
    block: String, // TODO: Find the CML type
    datum: Option<PlutusData>,
}

impl UTxO {
    pub fn new(
        tx_hash: TransactionHash,
        output_index: BigNum,
        amount: CMLValue,
        block: String,
        datum: Option<PlutusData>,
    ) -> Self {
        UTxO {
            tx_hash,
            output_index,
            amount,
            block,
            datum,
        }
    }

    pub fn tx_hash(&self) -> &TransactionHash {
        &self.tx_hash
    }

    pub fn output_index(&self) -> BigNum {
        self.output_index
    }

    pub fn amount(&self) -> &CMLValue {
        &self.amount
    }

    pub fn datum(&self) -> &Option<PlutusData> {
        &self.datum
    }
}

#[async_trait]
pub trait Ledger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>>;
    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String>;
}

impl<L, K, D, R> CMLLedgerCLient<L, K, D, R>
where
    L: Ledger,
    K: Keys,
    D: PlutusDataInterop,
    R: PlutusDataInterop,
{
    pub fn new(ledger: L, keys: K, network: u8) -> Self {
        CMLLedgerCLient {
            ledger,
            keys,
            network,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }

    async fn add_outputs_for_tx(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<D, R>,
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
            let mut output = TransactionOutput::new(&recp_addr, &cml_values);
            let res = if let UnbuiltOutput::Validator { datum, .. } = unbuilt_output {
                let data = datum.to_plutus_data();
                let data_hash = hash_plutus_data(&data);
                let cml_datum = CMLDatum::new_data_hash(&data_hash);
                output.set_datum(&cml_datum);
                let mut res = SingleOutputBuilderResult::new(&output);
                res.set_communication_datum(&data);
                res
            } else {
                SingleOutputBuilderResult::new(&output)
            };
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
    Datum: PlutusDataInterop + Send + Sync,
    Redeemer: PlutusDataInterop + Send + Sync,
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

                let utxos = todo!();
                // let utxos = bf_utxos
                //     .iter()
                //     .map(|utxo| bf_utxo_to_wallet_utxo(utxo, address))
                //     .collect();
                Ok(utxos)
            }
            Address::Script(addr_string) => {
                let script_addr = self
                    .keys
                    .addr_from_bech_32(addr_string)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let bf_utxos = self
                    .ledger
                    .get_utxos_for_addr(&script_addr)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let utxos = todo!();
                // let utxos = bf_utxos
                //     .iter()
                //     .map(|utxo| bf_utxo_to_validator_utxo(utxo, address))
                //     .collect();
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

        for (input, script) in tx.script_inputs() {
            let tx_hash_raw = input.id().tx_hash();
            let tx_hash = TransactionHash::from_hex(tx_hash_raw)
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
            let data = input.datum().unwrap(); // TODO
            let script_hex = script.script_hex();
            let script_bytes = hex::decode(&script_hex).map_err(as_failed_to_issue_tx)?;
            let v1 = PlutusV1Script::from_bytes(script_bytes)
                .map_err(|e| CMLLCError::Deserialize(e.to_string()))
                .map_err(as_failed_to_issue_tx)?; // TODO: version
            let cml_script = PlutusScript::from_v1(&v1);
            let script_witness = PlutusScriptWitness::from_script(cml_script.clone());
            let datum = data.to_plutus_data();
            let partial_witness = PartialPlutusWitness::new(&script_witness, &datum);

            let script_hash = cml_script.hash();
            let stake_cred = StakeCredential::from_scripthash(&script_hash);
            let enterprise_addr = EnterpriseAddress::new(self.network, &stake_cred);
            let cml_script_address = enterprise_addr.to_address();

            let required_signers = RequiredSigners::new();

            let script_input = TransactionInput::new(
                &tx_hash,                   // tx hash
                &input.id().index().into(), // index
            );
            let value = cmlvalues_from_values(input.values());
            let utxo_info = TransactionOutput::new(&cml_script_address, &value);
            let input_builder = SingleInputBuilder::new(&script_input, &utxo_info);
            let cml_input = input_builder
                .plutus_script(&partial_witness, &required_signers, &datum)
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
            tx_builder
                .add_input(&cml_input)
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
        }

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
        println!("{}", &tx.to_json().unwrap());
        // let submit_res = self
        //     .ledger
        //     .submit_transaction(&tx)
        //     .await
        //     .map_err(as_failed_to_issue_tx)?;
        // Ok(TxId::new(&submit_res))
        todo!()
    }
}

fn cmlvalues_from_values(values: &Values) -> CMLValue {
    todo!()
}
