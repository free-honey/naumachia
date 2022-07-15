use error::*;
use std::collections::HashMap;

mod error;

mod validator;

#[cfg(test)]
mod tests;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Address(String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}

pub type Policy = Option<Address>;
pub const ADA: Policy = None;

// TODO: Find max size stuff
pub struct Output {
    value: HashMap<Policy, u64>,
}

pub struct UnBuiltTransaction {
    inputs: Vec<Output>,
    outputs: Vec<Output>,
}

pub struct Transaction {
    inputs: Vec<Output>,
    outputs: Vec<Output>,
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

pub trait DataSource {}

pub trait TxBuilder {
    fn build(&self, unbuilt_tx: UnBuiltTransaction) -> Result<Transaction>;
}

pub trait TxIssuer {
    fn issue(&self, tx: Transaction) -> Result<()>;
}
