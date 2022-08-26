use crate::{
    config::{get_config, update_signer, write_config, Config},
    escrow_contract::{EscrowContract, EscrowDatum, EscrowEndpoint},
    handler::ActionHandler,
};

use clap::Parser;
use naumachia::address::PolicyId;
use naumachia::ledger_client::fake_address::FakeAddress;
use naumachia::{
    backend::Backend, error::Result as NauResult,
    ledger_client::local_persisted_ledger::LocalPersistedLedgerClient, ledger_client::LedgerClient,
    smart_contract::SmartContract,
};
use std::path::Path;

mod config;
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
    /// Check current signer's balance
    Balance,
    /// Redeem escrow contract for which signer is the receiver
    Claim { id: String },
    /// Create escrow contract for amount that only receiver can retrieve
    Escrow { amount: u64, receiver: String },
    /// List all active escrow contracts
    List,
    /// Change the signer to specified _simplified_ address, e.g. Alice, Bob, Charlie
    Signer { signer: String },
}

fn main() {
    let args = Args::parse();

    let logic = EscrowContract;

    let txo_record = setup_record();

    let backend = Backend::new(txo_record);
    let signer = backend.txo_record().signer();

    let contract = SmartContract::new(&logic, &backend);

    let handler = ActionHandler::new(contract);

    match args.action {
        ActionParams::Balance => {
            let balance = backend
                .ledger_client
                .balance_at_address(signer, &PolicyId::ADA);
            println!();
            println!("{}'s balance: {:?}", signer.to_str(), balance);
        }
        ActionParams::Claim { id } => handler.claim(&id).expect("unable to claim output"),
        ActionParams::Escrow { amount, receiver } => handler
            .escrow(amount, &receiver)
            .expect("unable to escrow funds"),
        ActionParams::List => handler
            .list()
            .expect("unable to list active escrow contracts"),
        ActionParams::Signer { signer } => {
            update_signer(&signer).expect("unable to update signer");
            println!();
            println!("Successfully updated signer to {:?}!", signer);
        }
    }
}

fn setup_record() -> LocalPersistedLedgerClient<EscrowDatum<FakeAddress>, ()> {
    let path = Path::new(".escrow_txo_record");
    let mut signer_str = "Alice".to_string();
    if let Some(config) = get_config() {
        signer_str = config.signer
    } else {
        let config = Config {
            signer: signer_str.clone(),
        };
        write_config(&config).expect("Could not write config file");
    };
    let signer = FakeAddress::new(&signer_str);
    let starting_amount = 10_000_000;
    LocalPersistedLedgerClient::init(path, signer, starting_amount).unwrap()
}
