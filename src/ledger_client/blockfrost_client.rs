use crate::ledger_client::{LedgerClient, TxORecordResult};
use crate::output::Output;
use crate::{Address, Transaction};
use std::marker::PhantomData;

pub mod blockfrost_http_client;

pub mod keys;

pub struct BlockFrostLedgerClient<Datum, Redeemer> {
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for BlockFrostLedgerClient<Datum, Redeemer> {
    fn signer(&self) -> &Address {
        todo!()
    }

    fn outputs_at_address(&self, _address: &Address) -> Vec<Output<Datum>> {
        todo!()
    }

    fn issue(&self, _tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()> {
        todo!()
    }
}
