use crate::{EscrowEndpoint, NauResult};
use naumachia::smart_contract::SmartContractTrait;

pub struct MockEscrowSmartContract;

impl SmartContractTrait for MockEscrowSmartContract {
    type Endpoint = EscrowEndpoint;
    type Lookup = ();
    type LookupResponse = ();

    fn hit_endpoint(&self, _endpoint: Self::Endpoint) -> NauResult<()> {
        Ok(())
    }

    fn lookup(&self, _lookup: Self::Lookup) -> NauResult<Self::LookupResponse> {
        todo!()
    }
}
