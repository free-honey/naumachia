use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::ledger_client::fake_address::FakeAddress;
use crate::values::Values;
use crate::{
    error::Result,
    ledger_client::{LedgerClient, LedgerClientError, TxORecordResult},
    output::Output,
    transaction::Transaction,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
struct Data<Datum> {
    signer: FakeAddress,
    outputs: Vec<Output<FakeAddress, Datum>>,
}

impl<Datum> Data<Datum> {
    pub fn new(signer: FakeAddress) -> Self {
        let outputs = Vec::new();
        Data { signer, outputs }
    }

    pub fn add_output(&mut self, output: Output<FakeAddress, Datum>) {
        self.outputs.push(output)
    }
}

pub struct LocalPersistedLedgerClient<Datum, Redeemer> {
    path: PathBuf,
    signer: FakeAddress,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

fn starting_output<Datum>(owner: &FakeAddress, amount: u64) -> Output<FakeAddress, Datum> {
    let id = Uuid::new_v4().to_string();
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    Output::Wallet {
        id,
        owner: owner.clone(),
        values,
    }
}

impl<Datum: Serialize + DeserializeOwned, Redeemer> LocalPersistedLedgerClient<Datum, Redeemer> {
    // TODO: Create builder
    pub fn init(path: &Path, signer: FakeAddress, starting_amount: u64) -> Result<Self> {
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

    fn update_outputs(&self, new_outputs: Vec<Output<FakeAddress, Datum>>) {
        let mut data = self.get_data();
        data.outputs = new_outputs;
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(&self.path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }
}

impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for LocalPersistedLedgerClient<Datum, Redeemer>
where
    Datum: Serialize + DeserializeOwned + Clone + PartialEq + Debug,
    Redeemer: Hash + Eq + Clone,
{
    type Address = FakeAddress;

    fn signer(&self) -> &FakeAddress {
        &self.signer
    }

    fn outputs_at_address(&self, address: &FakeAddress) -> Vec<Output<FakeAddress, Datum>> {
        let data = self.get_data();
        data.outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .collect()
    }

    fn issue(&self, tx: Transaction<FakeAddress, Datum, Redeemer>) -> TxORecordResult<()> {
        let mut my_outputs = self.get_data().outputs;
        for tx_i in tx.inputs() {
            let index = my_outputs.iter().position(|x| x == tx_i).ok_or_else(|| {
                LedgerClientError::FailedToRetrieveOutputWithId(tx_i.id().to_string())
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

    #[test]
    fn outputs_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = FakeAddress::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        let mut outputs = record.outputs_at_address(&signer);
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::ADA).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn balance_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = FakeAddress::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        let expected = starting_amount;
        let actual = record.balance_at_address(&signer, &PolicyId::ADA);
        assert_eq!(expected, actual);
    }

    #[test]
    fn issue() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = FakeAddress::new("alice");
        let starting_amount = 10_000_000;
        let record =
            LocalPersistedLedgerClient::<(), ()>::init(&path, signer.clone(), starting_amount)
                .unwrap();
        // let mut outputs = record.outputs_at_address(&signer);
        let first_output = record.outputs_at_address(&signer).pop().unwrap();
        let id = Uuid::new_v4().to_string();
        let owner = FakeAddress::new("bob");
        let new_output = Output::Wallet {
            id,
            owner: owner.clone(),
            values: first_output.values().clone(),
        };
        let tx: Transaction<FakeAddress, (), ()> = Transaction {
            inputs: vec![first_output],
            outputs: vec![new_output],
            redeemers: vec![],
            validators: Default::default(),
            minting: Default::default(),
            policies: Default::default(),
        };
        record.issue(tx).unwrap();
        let expected = starting_amount;
        let actual = record.balance_at_address(&owner, &PolicyId::ADA);
        assert_eq!(expected, actual)
    }
}
