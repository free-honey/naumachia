use crate::ledger_client::test_ledger_client::{InMemoryLCError, TestLedgerStorage};
use crate::ledger_client::LedgerClientError::FailedToIssueTx;
use crate::ledger_client::{LedgerClientError, LedgerClientResult};
use crate::output::Output;
use crate::Address;
use std::sync::{Arc, Mutex};

type MutableData<Datum> = Arc<Mutex<Vec<(Address, Output<Datum>)>>>;

#[derive(Debug)]
pub struct InMemoryStorage<Datum> {
    pub signer: Address,
    pub outputs: MutableData<Datum>,
}

#[async_trait::async_trait]
impl<Datum: Clone + Send + Sync + PartialEq> TestLedgerStorage<Datum> for InMemoryStorage<Datum> {
    async fn signer(&self) -> LedgerClientResult<Address> {
        Ok(self.signer.clone())
    }

    async fn outputs_by_count(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        let outputs = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| {
                LedgerClientError::FailedToRetrieveOutputsAt(address.clone(), Box::new(e))
            })?
            .iter()
            .cloned()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .take(count)
            .collect();
        Ok(outputs)
    }

    async fn all_outputs(&self, address: &Address) -> LedgerClientResult<Vec<Output<Datum>>> {
        let outputs = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| {
                LedgerClientError::FailedToRetrieveOutputsAt(address.clone(), Box::new(e))
            })?
            .iter()
            .cloned()
            .filter_map(|(a, o)| if &a == address { Some(o) } else { None })
            .collect();
        Ok(outputs)
    }

    async fn remove_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        let index = ledger_utxos
            .iter()
            .position(|(_, x)| x == output)
            .ok_or_else(|| {
                LedgerClientError::FailedToRetrieveOutputWithId(
                    output.id().clone(),
                    Box::new(InMemoryLCError::DuplicateInput),
                )
            })?;
        ledger_utxos.remove(index);
        Ok(())
    }

    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self
            .outputs
            .lock()
            .map_err(|e| InMemoryLCError::Mutex(format! {"{:?}", e}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        ledger_utxos.push((output.owner().clone(), output.clone()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use crate::transaction::ScriptVersion;
    use crate::{
        ledger_client::{
            test_ledger_client::{local_persisted_storage::starting_output, TestLedgerClient},
            LedgerClient,
        },
        output::UnbuiltOutput,
        PolicyId, UnbuiltTransaction,
    };

    #[tokio::test]
    async fn outputs_at_address() {
        let signer = Address::new("alice");
        let starting_amount = 10_000_000;
        let output = starting_output::<()>(&signer, starting_amount);
        let outputs = vec![(signer.clone(), output)];
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_in_memory(signer.clone(), outputs);
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
        let output = starting_output::<()>(&signer, starting_amount);
        let outputs = vec![(signer.clone(), output)];
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_in_memory(signer.clone(), outputs);
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
        let output = starting_output::<()>(&signer, starting_amount);
        let outputs = vec![(signer.clone(), output)];
        let record: TestLedgerClient<(), (), _> =
            TestLedgerClient::new_in_memory(signer.clone(), outputs);
        let first_output = record
            .all_outputs_at_address(&signer)
            .await
            .unwrap()
            .pop()
            .unwrap();
        let owner = Address::new("bob");
        let new_output = UnbuiltOutput::new_wallet(owner.clone(), first_output.values().clone());
        let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
            script_version: ScriptVersion::V1,
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
