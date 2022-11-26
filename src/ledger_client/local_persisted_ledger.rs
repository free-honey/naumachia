use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::File,
    hash::Hash,
    io::{Read, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;

use crate::ledger_client::in_memory_ledger::TestLedgerStorage;
use crate::ledger_client::LedgerClientError::FailedToIssueTx;
use crate::ledger_client::{build_outputs, new_wallet_output};
use crate::transaction::TxId;
use crate::values::Values;
use crate::{
    address::Address,
    error::Result,
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::Output,
    transaction::UnbuiltTransaction,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
struct LedgerData<Datum> {
    signer: Address,
    outputs: Vec<Output<Datum>>,
}

#[derive(Debug, Error)]
enum LocalPersistedLCError {
    #[error("Not enough input value available for outputs")]
    NotEnoughInputs,
    #[error("The same input is listed twice")]
    DuplicateInput, // TODO: WE don't need this once we dedupe
}

impl<Datum> LedgerData<Datum> {
    pub fn new(signer: Address) -> Self {
        let outputs = Vec::new();
        LedgerData { signer, outputs }
    }

    pub fn add_output(&mut self, output: Output<Datum>) {
        self.outputs.push(output)
    }
}

pub struct LocalPersistedLedgerClient<Datum, Redeemer> {
    path: PathBuf,
    signer: Address,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

pub fn starting_output<Datum>(owner: &Address, amount: u64) -> Output<Datum> {
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    Output::new_wallet(tx_hash, index, owner.clone(), values)
}

pub struct LocalPersistedStorage<Datum> {
    _datum: PhantomData<Datum>,
}

impl<Datum> TestLedgerStorage<Datum> for LocalPersistedStorage<Datum> {
    async fn signer(&self) -> LedgerClientResult<Address> {
        todo!()
    }

    async fn outputs_by_count(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        todo!()
    }

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        todo!()
    }

    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        todo!()
    }

    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        todo!()
    }
}

impl<Datum: Serialize + DeserializeOwned, Redeemer> LocalPersistedLedgerClient<Datum, Redeemer> {
    // TODO: Create builder
    pub fn init(path: &Path, signer: Address, starting_amount: u64) -> Result<Self> {
        if !path.exists() {
            let mut data = LedgerData::<Datum>::new(signer.clone());
            let output = starting_output(&signer, starting_amount);
            data.add_output(output); // TODO: Parameterize
            let serialized = serde_json::to_string(&data).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(&serialized.into_bytes()).unwrap();
        } else {
            // TODO: Ensure it is valid data?
        }
        let record = LocalPersistedLedgerClient {
            path: path.into(),
            signer,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(record)
    }

    fn get_data(&self) -> LedgerData<Datum> {
        let mut file = File::open(&self.path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read");
        serde_json::from_str(&contents).unwrap()
    }

    fn update_outputs(&self, new_outputs: Vec<Output<Datum>>) {
        let mut data = self.get_data();
        data.outputs = new_outputs;
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(&self.path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }
}

#[async_trait]
impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for LocalPersistedLedgerClient<Datum, Redeemer>
where
    Datum: Serialize + DeserializeOwned + Clone + PartialEq + Debug + Send + Sync,
    Redeemer: Hash + Eq + Clone + Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        Ok(self.signer.clone())
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let data = self.get_data();
        let outputs = data
            .outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .take(count)
            .collect();
        Ok(outputs)
    }

    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let data = self.get_data();
        let outputs = data
            .outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .collect();
        Ok(outputs)
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        // TODO: Have all matching Tx Id
        let signer = self.signer().await?;
        let mut combined_inputs = self.all_outputs_at_address(&signer).await?;
        tx.script_inputs()
            .iter()
            .for_each(|(input, _, _)| combined_inputs.push(input.clone())); // TODO: Check for dupes

        let mut total_input_value =
            combined_inputs
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });

        total_input_value.add_values(&tx.minting);

        let total_output_value =
            tx.unbuilt_outputs()
                .iter()
                .fold(Values::default(), |mut acc, utxo| {
                    acc.add_values(utxo.values());
                    acc
                });
        let maybe_remainder = total_input_value
            .try_subtract(&total_output_value)
            .map_err(|_| LocalPersistedLCError::NotEnoughInputs)
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;

        let mut ledger_utxos = self.get_data().outputs;

        for inputs in combined_inputs {
            let index = ledger_utxos
                .iter()
                .position(|x| x == &inputs)
                .ok_or_else(|| {
                    LedgerClientError::FailedToRetrieveOutputWithId(
                        inputs.id().clone(),
                        Box::new(LocalPersistedLCError::DuplicateInput),
                    )
                })?;
            ledger_utxos.remove(index);
        }

        let mut combined_outputs = Vec::new();
        if let Some(remainder) = maybe_remainder {
            combined_outputs.push(new_wallet_output(&signer, &remainder));
        }

        let built_outputs = build_outputs(tx.unbuilt_outputs);

        combined_outputs.extend(built_outputs);

        for output in combined_outputs {
            ledger_utxos.push(output.clone())
        }
        self.update_outputs(ledger_utxos);
        Ok(TxId::new("Not a real id"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use crate::output::UnbuiltOutput;
    use tempfile::TempDir;

    #[tokio::test]
    async fn outputs_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        let mut outputs = record.all_outputs_at_address(&signer).await.unwrap();
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn balance_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        let expected = starting_amount;
        let actual = record
            .balance_at_address(&signer, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn issue() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        let first_output = record
            .all_outputs_at_address(&signer)
            .await
            .unwrap()
            .pop()
            .unwrap();
        let owner = Address::new("bob");
        let new_output = UnbuiltOutput::new_wallet(owner.clone(), first_output.values().clone());
        let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
            script_inputs: vec![],
            unbuilt_outputs: vec![new_output],
            minting: Default::default(),
            policies: Default::default(),
        };
        record.issue(tx).await.unwrap();
        let expected_bob = starting_amount;
        let actual_bob = record
            .balance_at_address(&owner, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected_bob, actual_bob);

        let expected_alice = 0;
        let actual_alice = record
            .balance_at_address(&signer, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected_alice, actual_alice)
    }
}
