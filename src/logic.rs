use crate::txorecord::TxORecord;
use crate::{error::Result, UnBuiltTransaction};
use std::hash::Hash;

pub trait SCLogic {
    type Endpoint;
    type Lookup;
    type LookupResponse;
    type Datum: Clone;
    type Redeemer: Clone + PartialEq + Eq + Hash;

    fn handle_endpoint<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> Result<UnBuiltTransaction<Self::Datum, Self::Redeemer>>;

    fn lookup<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Lookup,
        txo_record: &Record,
    ) -> Result<Self::LookupResponse>;
}
