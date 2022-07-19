use crate::transaction::{Transaction, UnBuiltTransaction};
use error::*;
use std::collections::HashMap;

mod error;

pub mod transaction;
pub mod validator;

pub mod fakes;

#[cfg(test)]
mod tests;

// TODO: As of now, wallet, script, and policy addresses are the same. This is an
//       over-simplification in many ways. Wallet and address also need to be disambiguated.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Address(String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}

pub type Policy = Option<Address>;
pub const ADA: Policy = None;

pub type Value = (Policy, u64);

// TODO: Find max size instead of u64. It might not actually matter since we'll never be able to
//       select more than actually exists on chain. But maybe for minting?
#[derive(Clone, PartialEq, Debug)]
pub struct Output {
    owner: Address,
    values: HashMap<Policy, u64>,
}

pub trait SmartContract {
    type Endpoint;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> Result<UnBuiltTransaction>;

    fn hit_endpoint<D: DataSource, B: TxBuilder, I: TxIssuer>(
        endpoint: Self::Endpoint,
        source: &D,
        builder: &B,
        tx_issuer: &I,
    ) -> Result<()> {
        let unbuilt_tx = Self::handle_endpoint(endpoint, source)?;
        let tx = builder.build(unbuilt_tx)?;
        tx_issuer.issue(tx)?;
        Ok(())
    }
}

pub trait DataSource {
    fn me(&self) -> &Address;
}

// TODO: I have a suspiscion that a lot of this can be in a struct and just the input selection will
//       need to be injected? TBD.
pub trait TxBuilder {
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> Result<Transaction>;
}

pub trait TxIssuer {
    fn issue(&self, tx: Transaction) -> Result<()>;
}
