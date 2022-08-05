use crate::{error::Result, Address, Transaction, UnBuiltTransaction};
use std::hash::Hash;

pub trait SmartContract {
    type Endpoint;
    type Datum;
    type Redeemer: Clone + PartialEq + Eq + Hash;

    fn handle_endpoint(
        endpoint: Self::Endpoint,
        issuer: &Address,
    ) -> Result<UnBuiltTransaction<Self::Datum, Self::Redeemer>>;

    fn hit_endpoint<
        D: DataSource,
        B: TxBuilder<Self::Datum, Self::Redeemer>,
        I: TxIssuer<Self::Datum, Self::Redeemer>,
    >(
        endpoint: Self::Endpoint,
        source: &D,
        builder: &B,
        tx_issuer: &I,
    ) -> Result<()> {
        let issuer = source.me();
        let unbuilt_tx = Self::handle_endpoint(endpoint, &issuer)?;
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
pub trait TxBuilder<Datum, Redeemer: Clone + PartialEq + Eq + Hash> {
    fn build(
        &self,
        unbuilt_tx: UnBuiltTransaction<Datum, Redeemer>,
    ) -> Result<Transaction<Datum, Redeemer>>;
}

pub trait TxIssuer<Datum, Redeemer: Clone + PartialEq + Eq + Hash> {
    fn issue(&self, tx: Transaction<Datum, Redeemer>) -> Result<()>;
}
