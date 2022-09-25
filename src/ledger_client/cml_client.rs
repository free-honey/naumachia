use crate::{
    ledger_client::cml_client::issuance_helpers::{
        add_all_possible_utxos_for_selection, add_collateral, build_tx_for_signing,
        select_inputs_from_utxos, sign_tx, utxo_to_nau_utxo,
    },
    ledger_client::cml_client::plutus_data_interop::PlutusDataInterop,
    ledger_client::{
        cml_client::issuance_helpers::vasil_v1_tx_builder, LedgerClient, LedgerClientResult,
    },
    output::{Output, UnbuiltOutput},
    scripts::ValidatorCode,
    transaction::TxId,
    Address, UnbuiltTransaction,
};
use async_trait::async_trait;
use blockfrost_http_client::models::EvaluateTxResult;
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    address::{EnterpriseAddress, StakeCredential},
    builders::{
        input_builder::SingleInputBuilder,
        output_builder::SingleOutputBuilderResult,
        redeemer_builder::RedeemerWitnessKey,
        tx_builder::{ChangeSelectionAlgo, TransactionBuilder},
        witness_builder::{PartialPlutusWitness, PlutusScriptWitness},
    },
    crypto::{PrivateKey, TransactionHash},
    ledger::common::{hash::hash_plutus_data, value::BigNum, value::Value as CMLValue},
    plutus::{ExUnits, PlutusData, PlutusScript, PlutusV1Script, RedeemerTag},
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

#[derive(Debug)]
pub struct UTxO {
    tx_hash: TransactionHash,
    output_index: BigNum,
    amount: CMLValue,
    datum: Option<PlutusData>,
}

impl UTxO {
    pub fn new(
        tx_hash: TransactionHash,
        output_index: BigNum,
        amount: CMLValue,
        datum: Option<PlutusData>,
    ) -> Self {
        UTxO {
            tx_hash,
            output_index,
            amount,
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
    async fn calculate_ex_units(&self, tx: &CMLTransaction) -> Result<EvaluateTxResult>; // TODO: don't use bf types
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

    async fn add_outputs_for_tx<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
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
    async fn add_script_input<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        input: &Output<Datum>,
        redeemer: &Redeemer,
        script: &Box<dyn ValidatorCode<Datum, Redeemer> + '_>,
    ) -> LedgerClientResult<()> {
        let tx_hash_raw = input.id().tx_hash();
        let tx_hash = TransactionHash::from_hex(tx_hash_raw)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let script_hex = script.script_hex();
        let script_bytes = hex::decode(&script_hex).map_err(as_failed_to_issue_tx)?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| CMLLCError::Deserialize(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let cml_script = PlutusScript::from_v1(&v1);
        let script_witness = PlutusScriptWitness::from_script(cml_script.clone());
        let partial_witness =
            PartialPlutusWitness::new(&script_witness, &redeemer.to_plutus_data());

        let script_hash = cml_script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(self.network, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();

        let required_signers = RequiredSigners::new();

        let script_input = TransactionInput::new(
            &tx_hash,                   // tx hash
            &input.id().index().into(), // index
        );
        let value = input
            .values()
            .clone()
            .try_into()
            .map_err(as_failed_to_issue_tx)?;
        let utxo_info = TransactionOutput::new(&cml_script_address, &value);
        let input_builder = SingleInputBuilder::new(&script_input, &utxo_info);
        let data = input.datum().unwrap(); // TODO
        let datum = data.to_plutus_data();
        let cml_input = input_builder
            .plutus_script(&partial_witness, &required_signers, &datum)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        tx_builder
            .add_input(&cml_input)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Ok(())
    }

    async fn add_script_inputs<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for (input, redeemer, script) in tx.script_inputs() {
            self.add_script_input(tx_builder, input, redeemer, script)
                .await?
        }
        Ok(())
    }

    async fn update_ex_units(
        &self,
        tx_builder: &mut TransactionBuilder,
        my_address: &CMLAddress,
    ) -> LedgerClientResult<()> {
        let algo = ChangeSelectionAlgo::Default;
        let tx_redeemer_builder = tx_builder.build_for_evaluation(algo, &my_address).unwrap();
        let transaction = tx_redeemer_builder.draft_tx();
        let res = self.ledger.calculate_ex_units(&transaction).await.unwrap();
        for (index, spend) in res.get_spends().map_err(as_failed_to_issue_tx)? {
            tx_builder.set_exunits(
                &RedeemerWitnessKey::new(&RedeemerTag::new_spend(), &BigNum::from(index)), // TODO: How do I know which index?
                &ExUnits::new(&spend.memory().into(), &spend.steps().into()),
            );
        }
        Ok(())
    }

    async fn submit_tx(&self, tx: &CMLTransaction) -> LedgerClientResult<TxId> {
        let submit_res = self
            .ledger
            .submit_transaction(&tx)
            .await
            .map_err(as_failed_to_issue_tx)?;
        Ok(TxId::new(&submit_res))
    }

    async fn issue_v1_tx<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        &self,
        tx: UnbuiltTransaction<Datum, Redeemer>,
        my_utxos: Vec<UTxO>,
        my_address: CMLAddress,
        priv_key: PrivateKey,
    ) -> LedgerClientResult<TxId> {
        let mut tx_builder = vasil_v1_tx_builder()?;
        self.add_script_inputs(&mut tx_builder, &tx).await?;
        add_all_possible_utxos_for_selection(&mut tx_builder, &my_address, &my_utxos).await?;
        self.add_outputs_for_tx(&mut tx_builder, &tx).await?;
        add_collateral(&mut tx_builder, &my_address, &my_utxos).await?;
        select_inputs_from_utxos(&mut tx_builder).await?;
        self.update_ex_units(&mut tx_builder, &my_address).await?;
        let mut signed_tx_builder = build_tx_for_signing(&mut tx_builder, &my_address).await?;
        let tx = sign_tx(&mut signed_tx_builder, &priv_key).await?;
        let tx_id = self.submit_tx(&tx).await?;
        Ok(tx_id)
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
        let addr_string = match address {
            Address::Base(addr_string) => addr_string,
            Address::Script(addr_string) => addr_string,
            Address::Raw(_) => unimplemented!("Doesn't make sense here"),
        };
        let cml_addr = self
            .keys
            .addr_from_bech_32(addr_string)
            .await
            .map_err(as_failed_to_retrieve_by_address(address))?;

        let bf_utxos = self
            .ledger
            .get_utxos_for_addr(&cml_addr)
            .await
            .map_err(as_failed_to_retrieve_by_address(address))?;

        let utxos = bf_utxos
            .iter()
            .map(|utxo| utxo_to_nau_utxo(utxo, address))
            .collect();
        Ok(utxos)
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

        self.issue_v1_tx(tx, my_utxos, my_address, priv_key).await
    }
}
