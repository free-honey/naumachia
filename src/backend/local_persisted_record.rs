use crate::address::ADA;
use crate::{backend::TxORecord, output::Output, Address, Policy, Transaction};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::Result;

pub struct LocalPersistedRecord<Datum, Redeemer> {
    path: PathBuf,
    signer: Address,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

fn starting_output<Datum>(owner: &Address) -> Output<Datum> {
    let id = Uuid::new_v4().to_string();
    let mut values = HashMap::new();
    values.insert(ADA, 10_000_000);
    Output::Wallet {
        id,
        owner: owner.clone(),
        values,
    }
}

impl<Datum: Serialize, Redeemer> LocalPersistedRecord<Datum, Redeemer> {
    pub fn init(path: &Path, signer: Address) -> Result<Self> {
        if !path.exists() {
            let mut data = Data::<Datum>::new(signer.clone());
            let output = starting_output(&signer);
            data.add_output(output); // TODO: Parameterize
            let serialized = serde_json::to_string(&data).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(&serialized.into_bytes()).unwrap();
        } else {
            // maybe ensure it is valid data
        }
        let record = LocalPersistedRecord {
            path: path.into(),
            signer,
            _datum: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(record)
    }
}

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

impl<Datum: DeserializeOwned, Redeemer> TxORecord<Datum, Redeemer>
    for LocalPersistedRecord<Datum, Redeemer>
{
    fn signer(&self) -> &Address {
        &self.signer
    }

    fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        let mut file = File::open(&self.path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read");
        let data: Data<Datum> = serde_json::from_str(&contents).unwrap();
        data.outputs
            .into_iter()
            .filter(|o| o.owner() == address)
            .collect()
    }

    fn balance_at_address(&self, address: &Address, policy: &Policy) -> u64 {
        todo!()
    }

    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> crate::error::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[allow(non_snake_case)]
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn outputs_at_address() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("data");
        let signer = Address::new("me :)");
        let record = LocalPersistedRecord::<(), ()>::init(&path, signer.clone()).unwrap();
        let outputs = record.outputs_at_address(&signer);
        dbg!(outputs);
    }
}
