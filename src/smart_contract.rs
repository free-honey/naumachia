use crate::backend::{Backend, TxORecord};
use crate::error::Result;
use crate::logic::Logic;

#[derive(Debug)]
pub struct SmartContract<'a, SC, Record>
where
    SC: Logic,
    Record: TxORecord<SC::Datum, SC::Redeemer>,
{
    pub smart_contract: &'a SC,
    pub backend: &'a Backend<SC::Datum, SC::Redeemer, Record>,
}

impl<'a, SC, Record> SmartContract<'a, SC, Record>
where
    SC: Logic,
    Record: TxORecord<SC::Datum, SC::Redeemer>,
{
    pub fn new(
        smart_contract: &'a SC,
        backend: &'a Backend<SC::Datum, SC::Redeemer, Record>,
    ) -> Self {
        SmartContract {
            smart_contract,
            backend,
        }
    }

    pub fn hit_endpoint(&self, endpoint: SC::Endpoint) -> Result<()> {
        let unbuilt_tx = SC::handle_endpoint(endpoint, self.backend.signer())?;
        self.backend.process(unbuilt_tx)?;
        Ok(())
    }
}
