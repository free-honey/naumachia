use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    marker::PhantomData,
};
use tempfile::TempDir;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    address::Address,
    ledger_client::{test_ledger_client::TestLedgerStorage, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LedgerData<Datum> {
    signer: Address,
    outputs: Vec<Output<Datum>>,
}

#[derive(Debug, Error)]
enum LocalPersistedLCError {
    // #[error("Not enough input value available for outputs")]
    // NotEnoughInputs,
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

pub fn starting_output<Datum>(owner: &Address, amount: u64) -> Output<Datum> {
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    Output::new_wallet(tx_hash, index, owner.clone(), values)
}

pub struct LocalPersistedStorage<Datum> {
    tmp_dir: TempDir,
    _datum: PhantomData<Datum>,
}

const DATA: &str = "data";

impl<Datum: Serialize + DeserializeOwned> LocalPersistedStorage<Datum> {
    pub fn init(tmp_dir: TempDir, signer: Address, starting_amount: u64) -> Self {
        let path = tmp_dir.path().join(DATA);
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

        LocalPersistedStorage {
            tmp_dir,
            _datum: Default::default(),
        }
    }

    pub(crate) fn get_data(&self) -> LedgerData<Datum> {
        let path = self.tmp_dir.path().join(DATA);
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read");
        serde_json::from_str(&contents).unwrap()
    }

    fn update_outputs(&self, new_outputs: Vec<Output<Datum>>) {
        let path = self.tmp_dir.path().join(DATA);
        let mut data = self.get_data();
        data.outputs = new_outputs;
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(&path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }
}

#[async_trait::async_trait]
impl<Datum: Clone + Send + Sync + Serialize + DeserializeOwned + PartialEq> TestLedgerStorage<Datum>
    for LocalPersistedStorage<Datum>
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        let signer = self.get_data().signer;
        Ok(signer)
    }

    async fn outputs_by_count(
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

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        let data = self.get_data();
        let outputs = data
            .outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .collect();
        Ok(outputs)
    }

    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self.get_data().outputs;

        let index = ledger_utxos
            .iter()
            .position(|x| x == output)
            .ok_or_else(|| {
                LedgerClientError::FailedToRetrieveOutputWithId(
                    output.id().clone(),
                    Box::new(LocalPersistedLCError::DuplicateInput),
                )
            })?;
        ledger_utxos.remove(index);
        self.update_outputs(ledger_utxos);
        Ok(())
    }

    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self.get_data().outputs;
        ledger_utxos.push(output.to_owned());
        self.update_outputs(ledger_utxos);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::ledger_client::test_ledger_client::TestLedgerClient;
    use crate::ledger_client::LedgerClient;
    use crate::output::UnbuiltOutput;
    use crate::transaction::TransactionVersion;
    use crate::UnbuiltTransaction;

    #[tokio::test]
    async fn outputs_at_address() {
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_local_persisted(signer.clone(), starting_amount);
        let mut outputs = record.all_outputs_at_address(&signer).await.unwrap();
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn balance_at_address() {
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_local_persisted(signer.clone(), starting_amount);
        let expected = starting_amount;
        let actual = record
            .balance_at_address(&signer, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn issue() {
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_local_persisted(signer.clone(), starting_amount);
        let first_output = record
            .all_outputs_at_address(&signer)
            .await
            .unwrap()
            .pop()
            .unwrap();
        let owner = Address::new("bob");
        let new_output = UnbuiltOutput::new_wallet(owner.clone(), first_output.values().clone());
        let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
            script_version: TransactionVersion::V1,
            script_inputs: vec![],
            unbuilt_outputs: vec![new_output],
            minting: Default::default(),
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
