use clap::Parser;
use free_minting_contract::logic::{
    FreeMintingEndpoints,
    FreeMintingLogic,
};
use naumachia::{
    smart_contract::{
        SmartContract,
        SmartContractTrait,
    },
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
    let contract = SmartContract::new(logic, ledger_client);

    let tx_id = match args.action {
        ActionParams::Mint { amount } => contract
            .hit_endpoint(FreeMintingEndpoints::Mint { amount })
            .await
            .unwrap(),
    };
    println!("TxId: {:?}", tx_id);
}
