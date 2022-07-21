use crate::error::Result;
use crate::{Address, Transaction, UnBuiltTransaction};

pub trait SmartContract {
    type Endpoint;
    type Datum;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> Result<UnBuiltTransaction<Self::Datum>>;

    fn hit_endpoint<D: DataSource, B: TxBuilder<Self::Datum>, I: TxIssuer<Self::Datum>>(
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

// TODO: I have a suspicion that a lot of this can be in a struct and just the input selection will
//       need to be injected? TBD.
pub trait TxBuilder<Datum> {
    fn build(&self, unbuilt_tx: UnBuiltTransaction<Datum>) -> Result<Transaction<Datum>>;
}

pub trait TxIssuer<Datum> {
    fn issue(&self, tx: Transaction<Datum>) -> Result<()>;
}
