use crate::escrow_contract::EscrowDatum;
use crate::EscrowEndpoint;

use naumachia::address::ValidAddress;
use naumachia::ledger_client::fake_address::FakeAddress;
use naumachia::{error::Result as NauResult, output::Output, smart_contract::SmartContractTrait};

pub struct ActionHandler<SC: SmartContractTrait> {
    contract: SC,
}

impl<Address: ValidAddress, SC> ActionHandler<SC>
where
    SC: SmartContractTrait<
        Endpoint = EscrowEndpoint,
        Lookup = (),
        LookupResponse = Vec<Output<FakeAddress, EscrowDatum<Address>>>,
    >,
{
    pub fn new(contract: SC) -> Self {
        ActionHandler { contract }
    }

    pub fn escrow(&self, amount: u64, rcvr: &str) -> NauResult<()> {
        let receiver = FakeAddress::new(rcvr);
        let call = EscrowEndpoint::Escrow {
            amount,
            receiver: receiver.into(),
        };
        self.contract.hit_endpoint(call)?;
        println!();
        println!(
            "Successfully submitted escrow for {} Lovelace to {}!",
            amount, rcvr
        );
        Ok(())
    }

    pub fn claim(&self, output_id: &str) -> NauResult<()> {
        let call = EscrowEndpoint::Claim {
            output_id: output_id.to_string(),
        };
        self.contract.hit_endpoint(call)?;
        println!();
        println!("Successfully claimed output {}!", output_id);
        Ok(())
    }

    pub fn list(&self) -> NauResult<()> {
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
