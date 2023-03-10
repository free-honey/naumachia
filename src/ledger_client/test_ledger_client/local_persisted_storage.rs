use pallas_addresses::Address;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::Path;
use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    marker::PhantomData,
};
use tempfile::TempDir;
use thiserror::Error;

use crate::ledger_client::test_ledger_client::arbitrary_tx_id;
use crate::{
    ledger_client::{test_ledger_client::TestLedgerStorage, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LedgerData<Datum> {
    signer: String,
    outputs: Vec<Output<Datum>>,
    current_time: i64,
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
        LedgerData {
            signer: signer.to_bech32().expect("Already validated"),
            outputs,
            current_time: 0,
        }
    }

    pub fn add_output(&mut self, output: Output<Datum>) {
        self.outputs.push(output)
    }
}

pub fn starting_output<Datum>(owner: &Address, amount: u64) -> Output<Datum> {
    let tx_hash = arbitrary_tx_id().to_vec();
    let index = 0;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, amount);
    Output::new_wallet(tx_hash, index, owner.clone(), values)
}

pub struct LocalPersistedStorage<T: AsRef<Path>, Datum> {
    dir: T,
    _datum: PhantomData<Datum>,
}

const DATA: &str = "data";

impl<T: AsRef<Path>, Datum: Serialize + DeserializeOwned> LocalPersistedStorage<T, Datum> {
    pub fn init(dir: T, signer: Address, starting_amount: u64) -> Self {
        let path_ref: &Path = dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
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
            dir,
            _datum: Default::default(),
        }
    }

    pub fn with_dir(dir: T) -> Self {
        LocalPersistedStorage {
            dir,
            _datum: Default::default(),
        }
    }

    pub(crate) fn get_data(&self) -> LedgerData<Datum> {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read");
        serde_json::from_str(&contents).unwrap()
    }

    fn update_outputs(&self, new_outputs: Vec<Output<Datum>>) {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut data = self.get_data();
        data.outputs = new_outputs;
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }

    fn update_current_time(&self, posix_time: i64) -> LedgerClientResult<()> {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut data = self.get_data();
        data.current_time = posix_time;
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T, Datum> TestLedgerStorage<Datum> for LocalPersistedStorage<T, Datum>
where
    T: AsRef<Path> + Send + Sync,
    Datum: Clone + Send + Sync + Serialize + DeserializeOwned + PartialEq,
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        let signer = self.get_data().signer;
        Address::from_bech32(&signer).map_err(|e| LedgerClientError::BadAddress(Box::new(e)))
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
            .filter(|o| &o.owner() == address)
            .take(count)
            .collect();
        Ok(outputs)
    }

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        let data = self.get_data();
        let outputs = data
            .outputs
            .into_iter()
            .filter(|o| &o.owner() == address)
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

    async fn current_time(&self) -> LedgerClientResult<i64> {
        Ok(self.get_data().current_time)
    }

    async fn set_current_time(&mut self, posix_time: i64) -> LedgerClientResult<()> {
        self.update_current_time(posix_time).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[tokio::test]
    async fn outputs_at_address() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let starting_amount = 10_000_000;
        let tmp_dir = TempDir::new().unwrap();
        let storage =
            LocalPersistedStorage::<TempDir, ()>::init(tmp_dir, signer.clone(), starting_amount);
        let mut outputs = storage.all_outputs(&signer).await.unwrap();
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::Lovelace).unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn current_time() {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let starting_amount = 10_000_000;
        let tmp_dir = TempDir::new().unwrap();
        let mut storage =
            LocalPersistedStorage::<TempDir, ()>::init(tmp_dir, signer.clone(), starting_amount);
        let current_time = storage.current_time().await.unwrap();
        assert_eq!(current_time, 0);
        let new_time = 1000;
        storage.set_current_time(new_time).await.unwrap();
        let current_time = storage.current_time().await.unwrap();
        assert_eq!(current_time, new_time);
    }
}
