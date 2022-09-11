use crate::escrow_contract::EscrowDatum;
use crate::EscrowEndpoint;

use naumachia::output::OutputId;
use naumachia::{
    address::Address, error::Result as NauResult, output::Output,
    smart_contract::SmartContractTrait,
};

pub struct ActionHandler<SC: SmartContractTrait> {
    contract: SC,
}

impl<SC> ActionHandler<SC>
where
    SC: SmartContractTrait<
        Endpoint = EscrowEndpoint,
        Lookup = (),
        LookupResponse = Vec<Output<EscrowDatum>>,
    >,
{
    pub fn new(contract: SC) -> Self {
        ActionHandler { contract }
    }

    pub async fn escrow(&self, amount: u64, rcvr: &str) -> NauResult<()> {
        let receiver = Address::new(rcvr);
        let call = EscrowEndpoint::Escrow { amount, receiver };
        self.contract.hit_endpoint(call).await?;
        println!();
        println!(
            "Successfully submitted escrow for {} Lovelace to {}!",
            amount, rcvr
        );
        Ok(())
    }

    pub async fn claim(&self, tx_hash: &str, index: u64) -> NauResult<()> {
        let output_id = OutputId::new(tx_hash.to_string(), index);
        let call = EscrowEndpoint::Claim {
            output_id: output_id.clone(),
        };
        self.contract.hit_endpoint(call).await?;
        println!();
        println!("Successfully claimed output {:?}!", output_id);
        Ok(())
    }

    pub async fn list(&self) -> NauResult<()> {
        let outputs = self.contract.lookup(()).await?;
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
