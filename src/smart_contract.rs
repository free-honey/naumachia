use crate::backend::{Backend, TxORecord};
use crate::error::Result;
use crate::logic::Logic;

#[derive(Debug)]
pub struct SmartContract<SC, Record>
where
    SC: Logic,
    Record: TxORecord<SC::Datum, SC::Redeemer>,
{
    pub smart_contract: SC,
    pub backend: Backend<SC, SC::Datum, SC::Redeemer, Record>,
}

impl<SC, Record> SmartContract<SC, Record>
where
    SC: Logic,
    Record: TxORecord<SC::Datum, SC::Redeemer>,
{
    pub fn new(smart_contract: SC, backend: Backend<SC, SC::Datum, SC::Redeemer, Record>) -> Self {
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
