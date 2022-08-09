use crate::{
    backend::{Backend, TxORecord},
    error::Result,
    logic::SCLogic,
};

pub trait SmartContractTrait {
    type Endpoint;
    type Lookup;
    type LookupResponse;
    fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<()>;
    fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse>;
}

#[derive(Debug)]
pub struct SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: TxORecord<Logic::Datum, Logic::Redeemer>,
{
    pub smart_contract: &'a Logic,
    pub backend: &'a Backend<Logic::Datum, Logic::Redeemer, Record>,
}

impl<'a, Logic, Record> SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: TxORecord<Logic::Datum, Logic::Redeemer>,
{
    pub fn new(
        smart_contract: &'a Logic,
        backend: &'a Backend<Logic::Datum, Logic::Redeemer, Record>,
    ) -> Self {
        SmartContract {
            smart_contract,
            backend,
        }
    }
}

impl<'a, Logic, Record> SmartContractTrait for SmartContract<'a, Logic, Record>
where
    Logic: SCLogic,
    Record: TxORecord<Logic::Datum, Logic::Redeemer>,
{
    type Endpoint = Logic::Endpoint;
    type Lookup = Logic::Lookup;
    type LookupResponse = Logic::LookupResponse;

    fn hit_endpoint(&self, endpoint: Logic::Endpoint) -> Result<()> {
        let unbuilt_tx = Logic::handle_endpoint(endpoint, self.backend.txo_record())?;
        self.backend.process(unbuilt_tx)?;
        Ok(())
    }

    fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse> {
        Logic::lookup(lookup, self.backend.txo_record())
    }
}
