use pallas_addresses::{Address, Network};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::{
    fmt::Debug,
    fs::File,
    io::{Read, Write},
    marker::PhantomData,
};
use thiserror::Error;

use crate::ledger_client::test_ledger_client::arbitrary_tx_id;
use crate::output::OutputId;
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::{
    ledger_client::{test_ledger_client::TestLedgerStorage, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    PolicyId,
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LedgerData {
    active_signer_name: String,
    active_signer: String,
    signers: HashMap<String, String>,
    outputs: Vec<LDOutput>,
    current_time: i64,
    block_length: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct LDOutput {
    id: OutputId,
    owner: String,
    values: Values,
    datum: Option<PlutusData>,
}

impl<Datum: Clone + Into<PlutusData>> From<Output<Datum>> for LDOutput {
    fn from(output: Output<Datum>) -> Self {
        LDOutput {
            id: output.id().to_owned(),
            owner: output.owner().to_bech32().expect("Already validated"),
            values: output.values().clone(),
            datum: output.datum_plutus_data(),
        }
    }
}

impl<Datum> From<LDOutput> for Output<Datum> {
    fn from(value: LDOutput) -> Self {
        let LDOutput {
            id,
            owner,
            values,
            datum,
        } = value;
        let tx_hash = id.tx_hash().to_owned();
        let index = id.index();
        let owner = Address::from_bech32(&owner).unwrap(); // TODO: Unwrap
        if let Some(datum) = datum {
            Output::new_untyped_validator(tx_hash, index, owner, values, datum)
        } else {
            Output::new_wallet(tx_hash, index, owner, values)
        }
    }
}

#[derive(Debug, Error)]
enum LocalPersistedLCError {
    // #[error("Not enough input value available for outputs")]
    // NotEnoughInputs,
    #[error("The same input is listed twice")]
    DuplicateInput, // TODO: WE don't need this once we dedupe
}

impl LedgerData {
    pub fn new(signer_name: &str, signer_address: &Address, block_length: i64) -> Self {
        let outputs = Vec::new();
        let address_bech_32 = signer_address.to_bech32().expect("Already validated");
        let mut signers = HashMap::new();
        signers.insert(signer_name.to_string(), address_bech_32.clone());
        LedgerData {
            active_signer_name: signer_name.to_string(),
            active_signer: address_bech_32,
            signers,
            outputs,
            current_time: 0,
            block_length,
        }
    }

    pub fn signers(&self) -> Vec<String> {
        self.signers.keys().cloned().collect()
    }

    pub fn add_output<Datum: Clone + Into<PlutusData>>(&mut self, output: Output<Datum>) {
        self.outputs.push(output.into())
    }

    pub fn add_signer(&mut self, name: &str, address: &Address) {
        let address_bech_32 = address.to_bech32().expect("Already validated");
        self.signers.insert(name.to_string(), address_bech_32);
    }

    pub fn switch_signer(&mut self, name: &str) {
        self.active_signer_name = name.to_string();
        if let Some(address) = self.signers.get(name) {
            self.active_signer = address.to_string();
        } else {
            panic!("Signer not found"); // TODO
        };
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

// TODO: Make fallible!!!
impl<T, Datum> LocalPersistedStorage<T, Datum>
where
    T: AsRef<Path>,
    Datum: Clone + Into<PlutusData>,
{
    pub fn init(
        dir: T,
        signer_name: &str,
        signer: &Address,
        starting_amount: u64,
        starting_time: i64,
        block_length: i64,
    ) -> Self {
        let path_ref: &Path = dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        if !path.exists() {
            let mut data = LedgerData::new(signer_name, signer, block_length);
            data.current_time = starting_time;
            let output: Output<Datum> = starting_output(signer, starting_amount);
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

    pub fn load(dir: T) -> Self {
        LocalPersistedStorage {
            dir,
            _datum: Default::default(),
        }
    }

    pub(crate) fn get_data(&self) -> LedgerData {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read");
        serde_json::from_str(&contents).unwrap()
    }

    fn update_outputs(&self, new_outputs: Vec<LDOutput>) {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut data = self.get_data();
        data.outputs = new_outputs.into_iter().map(Into::into).collect();
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

    pub fn add_new_signer(&self, name: &str, address: &Address, starting_amount: u64) {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut data = self.get_data();
        let output: Output<Datum> = starting_output(address, starting_amount);
        data.add_output(output);
        data.add_signer(name, address);
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }

    pub fn active_signer_name(&self) -> String {
        let data = self.get_data();
        data.active_signer_name
    }

    pub fn get_signers(&self) -> Vec<String> {
        let data = self.get_data();
        data.signers()
    }

    pub fn switch_signer(&self, name: &str) {
        let path_ref: &Path = self.dir.as_ref();
        let path = path_ref.to_owned().join(DATA);
        let mut data = self.get_data();
        data.switch_signer(name);
        let serialized = serde_json::to_string(&data).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(&serialized.into_bytes()).unwrap();
    }
}

#[async_trait::async_trait]
impl<T, Datum> TestLedgerStorage<Datum> for LocalPersistedStorage<T, Datum>
where
    T: AsRef<Path> + Send + Sync,
    Datum: Clone + Send + Sync + PartialEq + Into<PlutusData> + TryFrom<PlutusData>,
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        let signer = self.get_data().active_signer;
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
            .map(Into::<Output<Datum>>::into)
            .filter(|o| &o.owner() == address)
            .take(count)
            .map(|output| output.with_typed_datum_if_possible())
            .collect();
        Ok(outputs)
    }

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        let data = self.get_data();
        let outputs = data
            .outputs
            .into_iter()
            .map(Into::<Output<Datum>>::into)
            .filter(|o| &o.owner() == address)
            .map(|output| output.with_typed_datum_if_possible())
            .collect();
        Ok(outputs)
    }

    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self.get_data().outputs;
        let sanitized_output = output.clone().into();

        let index = ledger_utxos
            .iter()
            .position(|x| x == &sanitized_output)
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
        let sanitized_output = output.clone().into();
        let mut ledger_utxos = self.get_data().outputs;
        ledger_utxos.push(sanitized_output);
        self.update_outputs(ledger_utxos);
        Ok(())
    }

    async fn current_time(&self) -> LedgerClientResult<i64> {
        Ok(self.get_data().current_time)
    }

    async fn set_current_time(&self, posix_time: i64) -> LedgerClientResult<()> {
        self.update_current_time(posix_time).unwrap();
        Ok(())
    }

    async fn get_block_length(&self) -> LedgerClientResult<i64> {
        let block_length = self.get_data().block_length;
        Ok(block_length)
    }

    async fn network(&self) -> LedgerClientResult<Network> {
        Ok(Network::Testnet)
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use tempfile::TempDir;

    const BLOCK_LENGTH: i64 = 1000;

    #[tokio::test]
    async fn outputs_at_address() {
        let signer_name = "Alice";
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let starting_amount = 10_000_000;
        let tmp_dir = TempDir::new().unwrap();
        let storage = LocalPersistedStorage::<TempDir, ()>::init(
            tmp_dir,
            signer_name,
            &signer,
            starting_amount,
            0,
            BLOCK_LENGTH,
        );
        let mut outputs = storage.all_outputs(&signer).await.unwrap();
        assert_eq!(outputs.len(), 1);
        let first_output = outputs.pop().unwrap();
        let expected = starting_amount;
        let actual = first_output.values().get(&PolicyId::Lovelace).unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn current_time() {
        let signer_name = "Alice";
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let starting_amount = 10_000_000;
        let tmp_dir = TempDir::new().unwrap();
        let storage = LocalPersistedStorage::<TempDir, ()>::init(
            tmp_dir,
            signer_name,
            &signer,
            starting_amount,
            0,
            BLOCK_LENGTH,
        );
        let current_time = storage.current_time().await.unwrap();
        assert_eq!(current_time, 0);
        let new_time = 1000;
        storage.set_current_time(new_time).await.unwrap();
        let current_time = storage.current_time().await.unwrap();
        assert_eq!(current_time, new_time);
    }

    #[tokio::test]
    async fn can_change_signer() {
        // Given
        let alice = "Alice";
        let alice_address = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let starting_amount = 10_000_000;
        let tmp_dir = TempDir::new().unwrap();
        let storage = LocalPersistedStorage::<TempDir, ()>::init(
            tmp_dir,
            alice,
            &alice_address,
            starting_amount,
            0,
            BLOCK_LENGTH,
        );
        let bob = "Bob";
        let bob_address = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        storage.add_new_signer(bob, &bob_address, starting_amount);

        // When
        let signer = storage.signer().await.unwrap();
        assert_eq!(signer, alice_address);

        // And
        storage.switch_signer(bob);
        let signer = storage.signer().await.unwrap();
        assert_eq!(signer, bob_address);
    }
}
