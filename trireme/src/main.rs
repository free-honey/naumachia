use crate::{
    init::init_impl,
    logic::{TriremeLogic, TriremeLookups, TriremeResponses},
};
use anyhow::Result;
use clap::Parser;
use naumachia::{
    backend::Backend,
    smart_contract::{SmartContract, SmartContractTrait},
    trireme_ledger_client::get_trireme_ledger_client_from_file,
};

mod init;
mod logic;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    /// Initialize Trireme configuration 🚣
    Init,
    /// Get ADA Balance
    Balance,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let logic = TriremeLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);
    match args.action {
        ActionParams::Init => init_impl().await?,
        ActionParams::Balance => {
            let res = contract.lookup(TriremeLookups::LovelaceBalance).await?;
            let ada = match res {
                TriremeResponses::LovelaceBalance(lovelace) => lovelace as f64 / 1_000_000.0,
            };
            println!("Balance: {:?} ADA", ada);
        }
    }
    Ok(())
}
