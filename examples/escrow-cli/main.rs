use crate::{escrow_contract::EscrowContract, handler::Handler};
use clap::Parser;
use escrow_contract::EscrowEndpoint;
use naumachia::address::ADA;
use naumachia::backend::TxORecord;
use naumachia::{
    address::Address, backend::local_persisted_record::LocalPersistedRecord, backend::Backend,
    error::Result as NauResult, smart_contract::SmartContract,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
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
    Claim { id: String },
    List,
    Signer { signer: String },
    Balance,
}

const CONFIG_PATH: &str = ".escrow_cli_config";

fn main() {
    let args = Args::parse();

    let logic = EscrowContract;
    let path = Path::new(".escrow_txo_record");
    let mut signer_str = "Alice".to_string();
    if let Some(config) = get_config() {
        signer_str = config.signer.to_string()
    } else {
        let config = Config {
            signer: signer_str.clone(),
        };
        write_config(&config).expect("Could not write config file");
    };
    let signer = Address::new(&signer_str);
    let starting_amount = 10_000_000;
    let txo_record = LocalPersistedRecord::init(&path, signer.clone(), starting_amount).unwrap();
    let backend = Backend::new(txo_record);
    let contract = SmartContract::new(&logic, &backend);

    let handler = Handler::new(contract);

    match args.action {
        ActionParams::Escrow { amount, receiver } => handler
            .escrow(amount, &receiver)
            .expect("unable to escrow funds"),
        ActionParams::Claim { id } => handler.claim(&id).expect("unable to claim output"),
        ActionParams::List => handler
            .list()
            .expect("unable to list active escrow contracts"),
        ActionParams::Signer { signer } => update_signer(signer).expect("unable to update signer"),
        ActionParams::Balance => {
            let balance = backend.txo_record.balance_at_address(&signer, &ADA);
            println!();
            println!("{}'s balance: {:?}", signer_str, balance);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    signer: String,
}

fn get_config() -> Option<Config> {
    let path = Path::new(CONFIG_PATH);
    if !path.exists() {
        None
    } else {
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read config file");
        let config = serde_json::from_str(&contents).unwrap();
        Some(config)
    }
}

fn update_signer(signer: String) -> Result<(), String> {
    let config = Config { signer };
    write_config(&config)?;
    Ok(())
}

fn write_config(config: &Config) -> Result<(), String> {
    let serialized = serde_json::to_string(config).unwrap();
    let mut file = File::create(CONFIG_PATH).unwrap();
    file.write_all(&serialized.into_bytes()).unwrap();
    Ok(())
}
