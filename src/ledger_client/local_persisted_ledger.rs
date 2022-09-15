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
use uuid::Uuid;

use crate::values::Values;
use crate::{
    address::Address,
    error::Result,
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::Output,
    transaction::Transaction,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
struct Data<Datum> {
    signer: Address,
    outputs: Vec<Output<Datum>>,
}

impl<Datum> Data<Datum> {
    pub fn new(signer: Address) -> Self {
        let outputs = Vec::new();
        Data { signer, outputs }
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

fn starting_output<Datum>(owner: &Address, amount: u64) -> Output<Datum> {
    let tx_hash = Uuid::new_v4().to_string();
    let index = 0;
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    Output::new_wallet(tx_hash, index, owner.clone(), values)
}

impl<Datum: Serialize + DeserializeOwned, Redeemer> LocalPersistedLedgerClient<Datum, Redeemer> {
    // TODO: Create builder
    pub fn init(path: &Path, signer: Address, starting_amount: u64) -> Result<Self> {
        if !path.exists() {
            let mut data = Data::<Datum>::new(signer.clone());
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

    fn get_data(&self) -> Data<Datum> {
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
    async fn signer(&self) -> LedgerClientResult<&Address> {
        Ok(&self.signer)
    }

    async fn outputs_at_address(
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

    async fn issue(&self, tx: Transaction<Datum, Redeemer>) -> LedgerClientResult<()> {
        let mut my_outputs = self.get_data().outputs;
        for tx_i in tx.inputs() {
            let index = my_outputs.iter().position(|x| x == tx_i).ok_or_else(|| {
                LedgerClientError::FailedToRetrieveOutputWithId(tx_i.id().clone())
            })?;
            my_outputs.remove(index);
        }

        for tx_o in tx.outputs() {
            my_outputs.push(tx_o.clone())
        }
        self.update_outputs(my_outputs);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
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
        let mut outputs = record.outputs_at_address(&signer).await.unwrap();
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
        // let mut outputs = record.outputs_at_address(&signer);
        let first_output = record
            .outputs_at_address(&signer)
            .await
            .unwrap()
            .pop()
            .unwrap();
        let tx_hash = Uuid::new_v4().to_string();
        let index = 0;
        let owner = Address::new("bob");
        let new_output =
            Output::new_wallet(tx_hash, index, owner.clone(), first_output.values().clone());
        let tx: Transaction<(), ()> = Transaction {
            script_inputs: vec![first_output],
            outputs: vec![new_output],
            redeemers: vec![],
            validators: Default::default(),
            minting: Default::default(),
            policies: Default::default(),
        };
        record.issue(tx).await.unwrap();
        let expected = starting_amount;
        let actual = record
            .balance_at_address(&owner, &PolicyId::ADA)
            .await
            .unwrap();
        assert_eq!(expected, actual)
    }
}
