use crate::{escrow_contract::EscrowContract, handler::Handler};
use clap::Parser;
use escrow_contract::EscrowEndpoint;
use naumachia::{
    address::Address, backend::local_persisted_record::LocalPersistedRecord, backend::Backend,
    error::Result as NauResult, smart_contract::SmartContract,
};
use std::path::Path;

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

    // let contract = MockEscrowSmartContract;
    let logic = EscrowContract;
    let path = Path::new(".escrow_txo_record");
    let signer = Address::new("Alice");
    let starting_amount = 10_000_000;
    let txo_record = LocalPersistedRecord::init(&path, signer, starting_amount).unwrap();
    let backend = Backend::new(txo_record);
    let contract = SmartContract::new(&logic, &backend);

    let handler = Handler::new(contract);

    match args.action {
        ActionParams::Escrow { amount, receiver } => handler
            .escrow(amount, &receiver)
            .expect("unable to escrow funds"),
        ActionParams::Claim { output } => handler.claim(&output).expect("unable to claim output"),
        ActionParams::List => handler
            .list()
            .expect("unable to list active escrow contracts"),
    }
}
