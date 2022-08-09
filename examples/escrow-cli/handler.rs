use crate::escrow_contract::EscrowDatum;
use crate::EscrowEndpoint;
use naumachia::address::Address;
use naumachia::output::Output;
use naumachia::smart_contract::SmartContractTrait;

pub struct Handler<SC: SmartContractTrait> {
    contract: SC,
}

impl<SC> Handler<SC>
where
    SC: SmartContractTrait<
        Endpoint = EscrowEndpoint,
        Lookup = (),
        LookupResponse = Vec<Output<EscrowDatum>>,
    >,
{
    pub fn new(contract: SC) -> Self {
        Handler { contract }
    }

    pub fn escrow(&self, amount: u64, rcvr: &str) -> Result<(), String> {
        let receiver = Address::new(rcvr);
        let call = EscrowEndpoint::Escrow { amount, receiver };
        self.contract.hit_endpoint(call)?;
        println!();
        println!(
            "Successfully submitted escrow for {} ADA to {}!",
            amount, rcvr
        );
        Ok(())
    }

    pub fn claim(&self, output_id: &str) -> Result<(), String> {
        let call = EscrowEndpoint::Claim {
            output_id: output_id.to_string(),
        };
        self.contract.hit_endpoint(call)?;
        println!();
        println!("Successfully claimed output {}!", output_id);
        Ok(())
    }

    pub fn list(&self) -> Result<(), String> {
        let outputs = self.contract.lookup(())?;
        println!();
        println!("Active contracts:");
        for utxo in outputs {
            println!(
                "id: {:?}, recipient: {:?}, values: {:?}",
                utxo.id(),
                utxo.datum().unwrap().receiver(),
                utxo.values()
            );
        }
        Ok(())
    }
}
