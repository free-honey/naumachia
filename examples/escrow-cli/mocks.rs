use crate::{Endpoint, NauResult};
use naumachia::smart_contract::SmartContractTrait;

pub struct MockEscrowSmartContract;

impl SmartContractTrait for MockEscrowSmartContract {
    type Endpoint = Endpoint;
    type Lookup = ();
    type LookupResponse = ();

    fn hit_endpoint(&self, _endpoint: Self::Endpoint) -> NauResult<()> {
        Ok(())
    }

    fn lookup(&self, _lookup: Self::Lookup) -> NauResult<Self::LookupResponse> {
        todo!()
    }
}
