use clap::Parser;
use free_minting_contract::logic::FreeMintingEndpoints;
use free_minting_contract::logic::FreeMintingLogic;
use naumachia::smart_contract::SmartContractTrait;
use naumachia::{
    backend::Backend, smart_contract::SmartContract,
    trireme_ledger_client::get_trireme_ledger_client_from_file,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    /// Mint amount
    Mint { amount: u64 },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let logic = FreeMintingLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);

    match args.action {
        ActionParams::Mint { amount } => contract
            .hit_endpoint(FreeMintingEndpoints::Mint { amount })
            .await
            .unwrap(),
    }
}
