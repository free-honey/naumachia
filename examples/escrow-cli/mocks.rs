use crate::escrow_contract::EscrowDatum;
use crate::{EscrowEndpoint, NauResult};
use async_trait::async_trait;
use naumachia::address::{Address, PolicyId};
use naumachia::output::Output;
use naumachia::smart_contract::SmartContractTrait;
use naumachia::values::Values;
use uuid::Uuid;

pub struct MockEscrowSmartContract;

#[async_trait]
impl SmartContractTrait for MockEscrowSmartContract {
    type Endpoint = EscrowEndpoint;
    type Lookup = ();
    type LookupResponse = Vec<Output<EscrowDatum>>;

    async fn hit_endpoint(&self, _endpoint: Self::Endpoint) -> NauResult<()> {
        Ok(())
    }

    async fn lookup(&self, _lookup: Self::Lookup) -> NauResult<Self::LookupResponse> {
        let mut values = Values::default();
        values.add_one_value(&PolicyId::ADA, 1234);
        let tx_hash = Uuid::new_v4().to_string();
        let index = 0;
        let owner = Address::new("someone");
        let output = Output::new_wallet(tx_hash, index, owner, values);
        let outputs = vec![output];
        Ok(outputs)
    }
}
