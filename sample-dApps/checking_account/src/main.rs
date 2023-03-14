use anyhow::Result;
use checking::{CheckingAccountEndpoints, CheckingAccountLogic};
use clap::Parser;
use naumachia::{
    backend::Backend,
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
    /// Create checking account
    Init {
        /// ADA Amount
        starting_ada: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        ActionParams::Init { starting_ada } => init_checking_account_impl(starting_ada).await?,
    }
    Ok(())
}

async fn hit_endpoint(endpoint: CheckingAccountEndpoints) -> Result<()> {
    let logic = CheckingAccountLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await?;
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);
    let res = contract.hit_endpoint(endpoint).await?;
    Ok(res)
}

async fn init_checking_account_impl(starting_ada: f64) -> Result<()> {
    let starting_lovelace = (starting_ada * 1_000_000.0) as u64; // TODO: Panic
    let endpoint = CheckingAccountEndpoints::InitAccount { starting_lovelace };
    hit_endpoint(endpoint).await
}
