use crate::escrow_contract::EscrowDatum;
use crate::{EscrowEndpoint, NauResult};
use naumachia::address::{Address, PolicyId};
use naumachia::output::Output;
use naumachia::smart_contract::SmartContractTrait;
use naumachia::values::Values;

pub struct MockEscrowSmartContract;

impl SmartContractTrait for MockEscrowSmartContract {
    type Endpoint = EscrowEndpoint;
    type Lookup = ();
    type LookupResponse = Vec<Output<EscrowDatum>>;

    fn hit_endpoint(&self, _endpoint: Self::Endpoint) -> NauResult<()> {
        Ok(())
    }

    fn lookup(&self, _lookup: Self::Lookup) -> NauResult<Self::LookupResponse> {
        let mut values = Values::default();
        values.add_one_value(&PolicyId::ADA, 1234);
        let output = Output::Wallet {
            id: "lolz".to_string(),
            owner: Address::new("someone"),
            values,
        };
        let outputs = vec![output];
        Ok(outputs)
    }
}
