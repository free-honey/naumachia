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
