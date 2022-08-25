use std::fmt::Debug;
use std::ops::Add;

use crate::address::ValidAddress;
use crate::{backend::Backend, error::Result, ledger_client::LedgerClient, logic::SCLogic};

pub trait SmartContractTrait {
    type Endpoint;
    type Lookup;
    type LookupResponse;
    fn hit_endpoint(&self, endpoint: Self::Endpoint) -> Result<()>;
    fn lookup(&self, lookup: Self::Lookup) -> Result<Self::LookupResponse>;
}

#[derive(Debug)]
pub struct SmartContract<'a, Address, Logic, LC>
where
    Logic: SCLogic<Address>,
    LC: LedgerClient<Logic::Datum, Logic::Redeemer, Address = Address>,
{
    pub logic: &'a Logic,
    pub backend: &'a Backend<LC::Address, Logic::Datum, Logic::Redeemer, LC>,
}

impl<'a, Address, Logic, LC> SmartContract<'a, Address, Logic, LC>
where
    Logic: SCLogic<Address>,
    LC: LedgerClient<Logic::Datum, Logic::Redeemer, Address = Address>,
{
    pub fn new(
        logic: &'a Logic,
        backend: &'a Backend<LC::Address, Logic::Datum, Logic::Redeemer, LC>,
    ) -> Self {
        SmartContract { logic, backend }
    }
}

impl<'a, Address, Logic, LC> SmartContractTrait for SmartContract<'a, Address, Logic, LC>
where
    Address: ValidAddress,
    Logic: SCLogic<Address> + Eq + Debug,
    LC: LedgerClient<Logic::Datum, Logic::Redeemer, Address = Address>,
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
        Ok(Logic::lookup(lookup, self.backend.txo_record())?)
    }
}
