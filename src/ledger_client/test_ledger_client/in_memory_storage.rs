use crate::ledger_client::test_ledger_client::{TestLCError, TestLedgerStorage};
use crate::ledger_client::LedgerClientError::FailedToIssueTx;
use crate::ledger_client::{LedgerClientError, LedgerClientResult};
use crate::output::Output;
use pallas_addresses::{Address, Network};
use std::sync::{Arc, Mutex};

/// A mutable, shared reference to a vector of `Output`s.
type MutableData<Datum> = Arc<Mutex<Vec<(Address, Output<Datum>)>>>;

/// Storage adapter for [`TestLedgerClient`] that is ephemeral and in-memory. This is useful
/// for Unit Tests and Integration Tests.
#[derive(Debug)]
pub struct InMemoryStorage<Datum> {
    /// The address of the signer key owned by this instance of the [`LedgerClient`]. This is a
    /// simplification of how Cardano works, but it's sufficient for testing.
    pub signer: Address,
    /// A list of all the outputs on this fake ledger.
    pub outputs: MutableData<Datum>,
    /// The current time on the ledger, or the time of the last "block" committed to the ledger.
    pub current_posix_time: Arc<Mutex<i64>>,
    /// The time between blocks on the ledger in seconds.
    pub block_length: i64,
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
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
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
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
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
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        let index = ledger_utxos
            .iter()
            .position(|(_, x)| x == output)
            .ok_or_else(|| {
                LedgerClientError::FailedToRetrieveOutputWithId(
                    output.id().clone(),
                    Box::new(TestLCError::DuplicateInput),
                )
            })?;
        ledger_utxos.remove(index);
        Ok(())
    }

    async fn add_output(&self, output: &Output<Datum>) -> LedgerClientResult<()> {
        let mut ledger_utxos = self
            .outputs
            .lock()
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        ledger_utxos.push((output.owner(), output.clone()));
        Ok(())
    }

    async fn current_time(&self) -> LedgerClientResult<i64> {
        let time = self
            .current_posix_time
            .lock()
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))?;
        Ok(*time)
    }

    async fn set_current_time(&self, posix_time: i64) -> LedgerClientResult<()> {
        *self
            .current_posix_time
            .lock()
            .map_err(|e| TestLCError::Mutex(format! {"{e:?}"}))
            .map_err(|e| FailedToIssueTx(Box::new(e)))? = posix_time;
        Ok(())
    }

    async fn get_block_length(&self) -> LedgerClientResult<i64> {
        let block_length = self.block_length;
        Ok(block_length)
    }

    async fn network(&self) -> LedgerClientResult<Network> {
        Ok(Network::Testnet)
    }
}
