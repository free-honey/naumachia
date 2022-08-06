use crate::handler::Handler;
use crate::mocks::MockEscrowSmartContract;
use clap::Parser;
use escrow_contract::EscrowEndpoint;
use naumachia::error::Result as NauResult;

mod escrow_contract;
mod handler;
mod mocks;

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
