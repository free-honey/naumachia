use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::{
    address::{Address, ADA},
    error::Result,
    output::Output,
    transaction::Transaction,
    txorecord::{TxORecord, TxORecordError, TxORecordResult},
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

pub struct LocalPersistedRecord<Datum, Redeemer> {
    path: PathBuf,
    signer: Address,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

fn starting_output<Datum>(owner: &Address, amount: u64) -> Output<Datum> {
    let id = Uuid::new_v4().to_string();
    let mut values = HashMap::new();
    values.insert(ADA, amount);
    Output::Wallet {
        id,
        owner: owner.clone(),
        values,
    }
}

impl<Datum: Serialize + DeserializeOwned, Redeemer> LocalPersistedRecord<Datum, Redeemer> {
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
        let record = LocalPersistedRecord {
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

impl<Datum, Redeemer> TxORecord<Datum, Redeemer> for LocalPersistedRecord<Datum, Redeemer>
where
    Datum: Serialize + DeserializeOwned + Clone + PartialEq + Debug,
    Redeemer: Hash + Eq + Clone,
{
    fn signer(&self) -> &Address {
        &self.signer
    }

    fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        let data = self.get_data();
        data.outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .collect()
    }

    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()> {
        let mut my_outputs = self.get_data().outputs;
        for tx_i in tx.inputs() {
            let index = my_outputs.iter().position(|x| x == tx_i).ok_or(
                TxORecordError::FailedToRetrieveOutputWithId(tx_i.id().to_string()),
            )?;
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

    #[test]
    fn outputs_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedRecord::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
        let mut outputs = record.outputs_at_address(&signer);
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&ADA).unwrap();
        assert_eq!(expected, *actual);
    }

    #[test]
    fn balance_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedRecord::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
        let expected = starting_amount;
        let actual = record.balance_at_address(&signer, &ADA);
        assert_eq!(expected, actual);
    }

    #[test]
    fn issue() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedRecord::<(), ()>::init(&path, signer.clone(), starting_amount).unwrap();
        // let mut outputs = record.outputs_at_address(&signer);
        let first_output = record.outputs_at_address(&signer).pop().unwrap();
        let id = Uuid::new_v4().to_string();
        let owner = Address::new("bob");
        let new_output = Output::Wallet {
            id,
            owner: owner.clone(),
            values: first_output.values().clone(),
        };
        let tx: Transaction<(), ()> = Transaction {
            inputs: vec![first_output],
            outputs: vec![new_output],
            redeemers: vec![],
            scripts: Default::default(),
        };
        record.issue(tx).unwrap();
        let expected = starting_amount;
        let actual = record.balance_at_address(&owner, &ADA);
        assert_eq!(expected, actual)
    }
}
