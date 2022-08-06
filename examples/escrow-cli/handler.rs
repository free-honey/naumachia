use crate::Endpoint;
use naumachia::address::Address;
use naumachia::smart_contract::SmartContractTrait;

pub struct Handler<SC: SmartContractTrait> {
    contract: SC,
}

impl<SC> Handler<SC>
where
    SC: SmartContractTrait<Endpoint = Endpoint>,
{
    pub fn new(contract: SC) -> Self {
        Handler { contract }
    }

    pub fn escrow(&self, amount: u64, rcvr: &str) -> Result<(), String> {
        let receiver = Address::new(rcvr);
        let call = Endpoint::Escrow { amount, receiver };
        self.contract.hit_endpoint(call)?;
        println!();
        println!(
            "Successfully submitted escrow for {} ADA to {}!",
            amount, rcvr
        );
        Ok(())
    }

    pub fn claim(&self, _output: &str) -> Result<(), String> {
        let _call = todo!("Need to add some ID field to outputs");
        // self.contract.hit_endpoint(_call)?;
        // println!();
        // println!("Successfully claimed output {}!", output);
        // Ok(())
    }
}
