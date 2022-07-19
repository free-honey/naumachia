use crate::error::Result;
use crate::{Address, Transaction, UnBuiltTransaction};

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
