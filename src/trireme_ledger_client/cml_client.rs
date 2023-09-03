use crate::trireme_ledger_client::cml_client::network_settings::NetworkSettings;
use crate::{
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::{Output, UnbuiltOutput},
    scripts::ValidatorCode,
    transaction::{TransactionVersion, TxId},
    trireme_ledger_client::cml_client::issuance_helpers::{
        cml_v1_script_from_nau_policy, cml_v2_script_from_nau_policy,
        cml_v2_script_from_nau_script, vasil_v2_tx_builder,
    },
    trireme_ledger_client::cml_client::{
        issuance_helpers::{
            add_collateral, build_tx_for_signing, cml_v1_script_from_nau_script, input_tx_hash,
            partial_script_witness, select_inputs_from_utxos, sign_tx,
            specify_utxos_available_for_input_selection, utxo_to_nau_utxo, vasil_v1_tx_builder,
        },
        plutus_data_interop::PlutusDataInterop,
    },
    UnbuiltTransaction,
};
use async_trait::async_trait;
use cardano_multiplatform_lib::{
    address::{Address as CMLAddress, BaseAddress, EnterpriseAddress, StakeCredential},
    builders::{
        input_builder::{InputBuilderResult, SingleInputBuilder},
        mint_builder::{MintBuilderResult, SingleMintBuilder},
        output_builder::SingleOutputBuilderResult,
        redeemer_builder::RedeemerWitnessKey,
        tx_builder::{ChangeSelectionAlgo, TransactionBuilder},
        witness_builder::{PartialPlutusWitness, PlutusScriptWitness},
    },
    crypto::{PrivateKey, TransactionHash},
    ledger::common::{hash::hash_plutus_data, value::BigNum, value::Int, value::Value as CMLValue},
    plutus::{ExUnits, PlutusData, PlutusScript, RedeemerTag},
    AssetName, Datum as CMLDatum, MintAssets, RequiredSigners, Transaction as CMLTransaction,
    TransactionInput, TransactionOutput,
};
use error::*;
use pallas_addresses::{Address, Network as CMLNetwork};
use std::time::UNIX_EPOCH;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, ops::Deref};

pub mod blockfrost_ledger;
pub mod error;
pub mod issuance_helpers;
pub mod key_manager;
pub mod network_settings;
pub mod ogmios_scrolls_ledger;
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
    network_settings: NetworkSettings,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

#[async_trait]
pub trait Keys {
    async fn base_addr(&self) -> Result<BaseAddress>;
    async fn private_key(&self) -> Result<PrivateKey>;
}

fn addr_from_bech_32(addr: &str) -> Result<CMLAddress> {
    let cml_address =
        CMLAddress::from_bech32(addr).map_err(|e| CMLLCError::JsError(e.to_string()))?;
    Ok(cml_address)
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

#[derive(Debug)]
pub struct ExecutionCost {
    execution_type: ExecutionType,
    memory: u64,
    steps: u64,
}

#[derive(Clone, Debug)]
pub enum ExecutionType {
    Spend,
    Mint,
    Certificate,
    Withdrawal,
}

impl ExecutionCost {
    pub fn new_spend(memory: u64, steps: u64) -> Self {
        let execution_type = ExecutionType::Spend;
        ExecutionCost {
            execution_type,
            memory,
            steps,
        }
    }

    pub fn new_mint(memory: u64, steps: u64) -> Self {
        let execution_type = ExecutionType::Mint;
        ExecutionCost {
            execution_type,
            memory,
            steps,
        }
    }

    pub fn new_certificate(memory: u64, steps: u64) -> Self {
        let execution_type = ExecutionType::Certificate;
        ExecutionCost {
            execution_type,
            memory,
            steps,
        }
    }

    pub fn new_withdrawal(memory: u64, steps: u64) -> Self {
        let execution_type = ExecutionType::Withdrawal;
        ExecutionCost {
            execution_type,
            memory,
            steps,
        }
    }

    pub fn execution_type(&self) -> ExecutionType {
        self.execution_type.clone()
    }

    pub fn memory(&self) -> u64 {
        self.memory
    }
    pub fn steps(&self) -> u64 {
        self.steps
    }
}

#[async_trait]
pub trait Ledger {
    async fn last_block_time_secs(&self) -> Result<i64>;
    async fn get_utxos_for_addr(&self, addr: &CMLAddress, count: usize) -> Result<Vec<UTxO>>;
    async fn get_all_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<UTxO>>;
    async fn calculate_ex_units(&self, tx: &CMLTransaction) -> Result<HashMap<u64, ExecutionCost>>;
    async fn submit_transaction(&self, tx: &CMLTransaction) -> Result<String>;
}

impl<L, K, D, R> CMLLedgerCLient<L, K, D, R>
where
    L: Ledger,
    K: Keys,
    D: PlutusDataInterop,
    R: PlutusDataInterop,
{
    pub fn new(ledger: L, keys: K, network_settings: NetworkSettings) -> Self {
        CMLLedgerCLient {
            ledger,
            keys,
            network_settings,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }

    async fn add_outputs_for_tx<Datum: PlutusDataInterop + Debug, Redeemer: PlutusDataInterop>(
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
            let recp_addr =
                addr_from_bech_32(&recipient.to_string()).map_err(as_failed_to_issue_tx)?;
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

    async fn cml_script_address(&self, cml_script: &PlutusScript) -> CMLAddress {
        let script_hash = cml_script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(self.network_settings.network(), &stake_cred);
        enterprise_addr.to_address()
    }

    async fn build_v1_cml_script_input<
        Datum: PlutusDataInterop + Clone,
        Redeemer: PlutusDataInterop,
    >(
        &self,
        input: &Output<Datum>,
        redeemer: &Redeemer,
        script: &(dyn ValidatorCode<Datum, Redeemer> + '_),
    ) -> LedgerClientResult<InputBuilderResult> {
        let tx_hash = input_tx_hash(input).await?;
        let cml_script = cml_v1_script_from_nau_script(script).await?;
        let partial_witness = partial_script_witness(&cml_script, redeemer).await;
        let cml_script_address = self.cml_script_address(&cml_script).await;
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
        let maybe_data: Option<Datum> = input.datum().to_owned().into();
        let data = maybe_data.ok_or(LedgerClientError::NoDatumOnScriptInput)?;
        let datum = data.to_plutus_data();
        let cml_input = input_builder
            .plutus_script(&partial_witness, &required_signers, &datum)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Ok(cml_input)
    }

    async fn build_v2_cml_script_input<
        Datum: PlutusDataInterop + Clone,
        Redeemer: PlutusDataInterop,
    >(
        &self,
        input: &Output<Datum>,
        redeemer: &Redeemer,
        script: &(dyn ValidatorCode<Datum, Redeemer> + '_),
    ) -> LedgerClientResult<InputBuilderResult> {
        let tx_hash = input_tx_hash(input).await?;
        let cml_script = cml_v2_script_from_nau_script(script).await?;
        let partial_witness = partial_script_witness(&cml_script, redeemer).await;
        let cml_script_address = self.cml_script_address(&cml_script).await;
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
        let maybe_data: Option<Datum> = input.datum().to_owned().into();
        let data = maybe_data.ok_or(LedgerClientError::NoDatumOnScriptInput)?;
        let datum = data.to_plutus_data();
        let cml_input = input_builder
            .plutus_script(&partial_witness, &required_signers, &datum)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Ok(cml_input)
    }

    async fn add_v1_script_input<Datum: PlutusDataInterop + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        input: &Output<Datum>,
        redeemer: &Redeemer,
        script: &(dyn ValidatorCode<Datum, Redeemer> + '_),
    ) -> LedgerClientResult<()> {
        let cml_input = self
            .build_v1_cml_script_input(input, redeemer, script)
            .await?;
        tx_builder
            .add_input(&cml_input)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Ok(())
    }

    async fn add_v2_script_input<Datum: PlutusDataInterop + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        input: &Output<Datum>,
        redeemer: &Redeemer,
        script: &(dyn ValidatorCode<Datum, Redeemer> + '_),
    ) -> LedgerClientResult<()> {
        let cml_input = self
            .build_v2_cml_script_input(input, redeemer, script)
            .await?;
        tx_builder
            .add_input(&cml_input)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        Ok(())
    }

    async fn add_tokens_for_v1_minting<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for (amount, asset_name, redeemer, policy) in tx.minting.iter() {
            let script = cml_v1_script_from_nau_policy(policy.deref()).await?;
            let mint_builder_res = self
                .build_mint_res(*amount, asset_name, redeemer, script)
                .await?;
            tx_builder.add_mint(&mint_builder_res);
        }
        Ok(())
    }

    async fn add_tokens_for_v2_minting<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for (amount, asset_name, redeemer, policy) in tx.minting.iter() {
            let script = cml_v2_script_from_nau_policy(policy.deref()).await?;
            let mint_builder_res = self
                .build_mint_res(*amount, asset_name, redeemer, script)
                .await?;
            tx_builder.add_mint(&mint_builder_res);
        }
        Ok(())
    }

    async fn build_mint_res<Redeemer: PlutusDataInterop>(
        &self,
        amount: u64,
        asset_name: &Option<String>,
        redeemer: &Redeemer,
        script: PlutusScript,
    ) -> LedgerClientResult<MintBuilderResult> {
        let inner_key = if let Some(name) = asset_name {
            name.as_bytes().to_vec()
        } else {
            Vec::new()
        };
        let key = AssetName::new(inner_key)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let big_num = BigNum::from(amount);
        let value = Int::new(&big_num);
        let mint_assets = MintAssets::new_from_entry(&key, value);
        let mint_builder = SingleMintBuilder::new(&mint_assets);
        let script_witness = PlutusScriptWitness::from_script(script);
        let redeemer = redeemer.to_plutus_data();
        let partial_witness = PartialPlutusWitness::new(&script_witness, &redeemer);
        let required_signers = RequiredSigners::new();
        let res = mint_builder.plutus_script(&partial_witness, &required_signers);
        Ok(res)
    }

    async fn add_v1_script_inputs<Datum: PlutusDataInterop + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for (input, redeemer, script) in tx.script_inputs() {
            self.add_v1_script_input(tx_builder, input, redeemer, script.deref())
                .await?
        }
        Ok(())
    }

    async fn add_v2_script_inputs<Datum: PlutusDataInterop + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for (input, redeemer, script) in tx.script_inputs() {
            self.add_v2_script_input(tx_builder, input, redeemer, script.deref())
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
        let tx_redeemer_builder = tx_builder
            .build_for_evaluation(algo, my_address)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        let transaction = tx_redeemer_builder.draft_tx();
        println!("{}", &transaction.to_json().unwrap());
        let res = self
            .ledger
            .calculate_ex_units(&transaction)
            .await
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(as_failed_to_issue_tx)?;
        for (index, spend) in res.iter() {
            let tag = match spend.execution_type {
                ExecutionType::Spend => RedeemerTag::new_spend(),
                ExecutionType::Mint => RedeemerTag::new_mint(),
                ExecutionType::Withdrawal => RedeemerTag::new_reward(),
                ExecutionType::Certificate => RedeemerTag::new_cert(),
            };
            tx_builder.set_exunits(
                &RedeemerWitnessKey::new(&tag, &BigNum::from(*index)),
                &ExUnits::new(&spend.memory().into(), &spend.steps().into()),
            );
        }
        Ok(())
    }

    async fn submit_tx(&self, tx: &CMLTransaction) -> LedgerClientResult<TxId> {
        let submit_res = self
            .ledger
            .submit_transaction(tx)
            .await
            .map_err(as_failed_to_issue_tx)?;
        Ok(TxId::new(&submit_res))
    }

    async fn issue_v1_tx<Datum: PlutusDataInterop + Debug + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx: UnbuiltTransaction<Datum, Redeemer>,
        my_utxos: Vec<UTxO>,
        my_address: CMLAddress,
        priv_key: PrivateKey,
    ) -> LedgerClientResult<TxId> {
        let mut tx_builder = vasil_v1_tx_builder()?;
        self.add_v1_script_inputs(&mut tx_builder, &tx).await?;
        self.add_tokens_for_v1_minting(&mut tx_builder, &tx).await?;
        specify_utxos_available_for_input_selection(&mut tx_builder, &my_address, &my_utxos)
            .await?;
        self.add_outputs_for_tx(&mut tx_builder, &tx).await?;
        add_collateral(&mut tx_builder, &my_address, &my_utxos).await?;
        select_inputs_from_utxos(&mut tx_builder).await?;
        self.update_ex_units(&mut tx_builder, &my_address).await?;
        let mut signed_tx_builder = build_tx_for_signing(&mut tx_builder, &my_address).await?;
        let tx = sign_tx(&mut signed_tx_builder, &priv_key).await?;
        let tx_id = self.submit_tx(&tx).await?;
        Ok(tx_id)
    }

    async fn issue_v2_tx<Datum: PlutusDataInterop + Debug + Clone, Redeemer: PlutusDataInterop>(
        &self,
        tx: UnbuiltTransaction<Datum, Redeemer>,
        my_utxos: Vec<UTxO>,
        my_address: CMLAddress,
        priv_key: PrivateKey,
    ) -> LedgerClientResult<TxId> {
        let mut tx_builder = vasil_v2_tx_builder()?;
        self.set_valid_range(&mut tx_builder, &tx).await?;
        self.add_v2_script_inputs(&mut tx_builder, &tx).await?;
        self.add_tokens_for_v2_minting(&mut tx_builder, &tx).await?;
        specify_utxos_available_for_input_selection(&mut tx_builder, &my_address, &my_utxos)
            .await?;
        self.add_specific_inputs(&mut tx_builder, &tx).await?;
        self.add_outputs_for_tx(&mut tx_builder, &tx).await?;
        add_collateral(&mut tx_builder, &my_address, &my_utxos).await?;
        select_inputs_from_utxos(&mut tx_builder).await?;
        self.update_ex_units(&mut tx_builder, &my_address).await?;
        let mut signed_tx_builder = build_tx_for_signing(&mut tx_builder, &my_address).await?;
        let tx = sign_tx(&mut signed_tx_builder, &priv_key).await?;
        let tx_id = self.submit_tx(&tx).await?;
        Ok(tx_id)
    }

    async fn set_valid_range<Datum: PlutusDataInterop + Debug, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        let (lower, upper) = tx.valid_range;
        if let Some(posix) = lower {
            let slot = self.network_settings.slot_from_posix(posix).ok_or(
                LedgerClientError::ValidityRange(format!(
                    "Invalid lower bounds; Must be after starting slot time: {:?}",
                    self.network_settings.starting_slot_time()
                )),
            )?;
            tx_builder.set_validity_start_interval(&slot);
        }
        if let Some(posix) = upper {
            let slot = self.network_settings.slot_from_posix(posix).ok_or(
                LedgerClientError::ValidityRange(format!(
                    "Invalid upper bounds; Must be after starting slot time: {:?}",
                    self.network_settings.starting_slot_time()
                )),
            )?;
            tx_builder.set_ttl(&slot);
        }
        Ok(())
    }

    async fn add_specific_inputs<Datum: PlutusDataInterop + Debug, Redeemer: PlutusDataInterop>(
        &self,
        tx_builder: &mut TransactionBuilder,
        tx: &UnbuiltTransaction<Datum, Redeemer>,
    ) -> LedgerClientResult<()> {
        for specific_input in &tx.specific_wallet_inputs {
            let transaction_id = input_tx_hash(specific_input).await?;
            let index = specific_input.id().index().into();
            let input = TransactionInput::new(&transaction_id, &index);
            let address = addr_from_bech_32(&specific_input.owner().to_string())
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
            let amount = specific_input
                .values()
                .clone()
                .try_into()
                .map_err(as_failed_to_issue_tx)?;
            let utxo_info = TransactionOutput::new(&address, &amount);
            let input_builder = SingleInputBuilder::new(&input, &utxo_info);
            let res = input_builder
                .payment_key()
                .map_err(|e| CMLLCError::JsError(e.to_string()))
                .map_err(as_failed_to_issue_tx)?;
            tx_builder
                .add_input(&res)
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
    Datum: PlutusDataInterop + Send + Sync + Debug + Clone,
    Redeemer: PlutusDataInterop + Send + Sync,
{
    async fn signer_base_address(&self) -> LedgerClientResult<Address> {
        let base_addr = self
            .keys
            .base_addr()
            .await
            .map_err(|e| LedgerClientError::BaseAddress(Box::new(e)))?;
        let addr_string = base_addr
            .to_address()
            .to_bech32(None)
            .map_err(|e| CMLLCError::JsError(e.to_string()))
            .map_err(|e| LedgerClientError::BaseAddress(Box::new(e)))?;
        let signer_addr = Address::from_bech32(&addr_string)
            .map_err(|e| LedgerClientError::BaseAddress(Box::new(e)))?;
        Ok(signer_addr)
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let addr_string = address
            .to_bech32()
            .map_err(|e| LedgerClientError::BadAddress(Box::new(e)))?;
        let cml_addr =
            addr_from_bech_32(&addr_string).map_err(as_failed_to_retrieve_by_address(address))?;

        let bf_utxos = self
            .ledger
            .get_utxos_for_addr(&cml_addr, count)
            .await
            .map_err(as_failed_to_retrieve_by_address(address))?;

        let utxos = bf_utxos
            .iter()
            .map(|utxo| utxo_to_nau_utxo(utxo, address))
            .collect::<LedgerClientResult<Vec<_>>>()?;

        Ok(utxos)
    }

    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let addr_string = address.to_bech32().expect("Already Validated");
        let cml_addr =
            addr_from_bech_32(&addr_string).map_err(as_failed_to_retrieve_by_address(address))?;

        let bf_utxos = self
            .ledger
            .get_all_utxos_for_addr(&cml_addr)
            .await
            .map_err(as_failed_to_retrieve_by_address(address))?;

        let utxos = bf_utxos
            .iter()
            .map(|utxo| utxo_to_nau_utxo(utxo, address))
            .collect::<LedgerClientResult<Vec<_>>>()?;

        Ok(utxos)
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        let my_address = self
            .keys
            .base_addr()
            .await
            .map_err(as_failed_to_issue_tx)?
            .to_address();
        let priv_key = self
            .keys
            .private_key()
            .await
            .map_err(as_failed_to_issue_tx)?;

        let my_utxos = self
            .ledger
            .get_all_utxos_for_addr(&my_address)
            .await
            .map_err(as_failed_to_issue_tx)?;

        match tx.script_version {
            TransactionVersion::V1 => self.issue_v1_tx(tx, my_utxos, my_address, priv_key).await,
            TransactionVersion::V2 => self.issue_v2_tx(tx, my_utxos, my_address, priv_key).await,
        }
    }

    async fn network(&self) -> LedgerClientResult<CMLNetwork> {
        let network = match self.network_settings.network() {
            0 => CMLNetwork::Testnet,
            1 => CMLNetwork::Mainnet,
            _ => CMLNetwork::Other(self.network_settings.network()),
        };
        Ok(network)
    }

    async fn last_block_time_secs(&self) -> LedgerClientResult<i64> {
        self.ledger
            .last_block_time_secs()
            .await
            .map_err(as_failed_to_get_block_time)
    }

    async fn current_time_secs(&self) -> LedgerClientResult<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| LedgerClientError::CurrentTime(Box::new(e)))?
            .as_secs()
            .try_into()
            .expect("This should never be bigger than i64 in our lifetimes :)");
        Ok(now)
    }
}
