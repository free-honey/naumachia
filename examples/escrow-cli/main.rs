use clap::Parser;
use escrow_contract::Endpoint;
use naumachia::{address::Address, error::Result as NauResult, smart_contract::SmartContractTrait};

mod escrow_contract;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    Escrow { amount: u64, receiver: String },
    Claim { output: String },
    List,
}

fn main() {
    let args = Args::parse();

    let contract = MockEscrowSmartContract;

    let handler = Handler::new(contract);

    match args.action {
        ActionParams::Escrow { amount, receiver } => handler
            .escrow(amount, &receiver)
            .expect("unable to escrow funds"),
        ActionParams::Claim { output } => handler.claim(&output).expect("unable to claim output"),
        _ => todo!(),
    }
}

struct Handler<SC: SmartContractTrait> {
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

struct MockEscrowSmartContract;

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
