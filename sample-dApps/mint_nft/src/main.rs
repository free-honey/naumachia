use clap::Parser;
use mint_nft::logic::{MintNFTEndpoints, MintNFTLogic};
use naumachia::{
    backend::Backend,
    output::OutputId,
    smart_contract::{SmartContract, SmartContractTrait},
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
    /// Mint single NFT
    Mint,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = MintNFTLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);

    match args.action {
        ActionParams::Mint => contract.hit_endpoint(MintNFTEndpoints::Mint).await.unwrap(),
    }
}
