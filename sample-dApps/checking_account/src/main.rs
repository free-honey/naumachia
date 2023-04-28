use anyhow::Result;
use checking::{
    CheckingAccountEndpoints, CheckingAccountLogic, CheckingAccountLookupResponses,
    CheckingAccountLookups,
};
use clap::Parser;
use naumachia::scripts::context::PubKeyHash;
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
    /// Lookup all checking accounts owned by me
    MyAccounts,
    /// Starts a dialogue to add a puller for a checking account
    AddPuller,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        ActionParams::Init { starting_ada } => init_checking_account_impl(starting_ada).await?,
        ActionParams::MyAccounts => my_account_impl().await?,
        ActionParams::AddPuller => add_puller_impl().await?,
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

async fn run_lookup(lookup: CheckingAccountLookups) -> Result<CheckingAccountLookupResponses> {
    let logic = CheckingAccountLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await?;
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);
    let res = contract.lookup(lookup).await?;
    Ok(res)
}

// TODO: Give some feedback?
async fn init_checking_account_impl(starting_ada: f64) -> Result<()> {
    let starting_lovelace = (starting_ada * 1_000_000.0) as u64; // TODO: Panic
    let endpoint = CheckingAccountEndpoints::InitAccount { starting_lovelace };
    hit_endpoint(endpoint).await
}

async fn my_account_impl() -> Result<()> {
    let lookup = CheckingAccountLookups::MyAccounts;
    let res = run_lookup(lookup).await?;
    match res {
        CheckingAccountLookupResponses::MyAccounts(accounts) => {
            accounts.iter().for_each(|account| {
                println!("==========================================");
                println!("{:?} ADA", account.balance_ada);
                if let Some(nft) = &account.nft {
                    println!("attached to nft: {:?}", nft);
                }
                for puller in &account.pullers {
                    let pkh = hex::encode(puller.puller.bytes());
                    let amount = puller.amount_lovelace / 1_000_000;
                    let period = puller.period;
                    let next_pull = puller.next_pull;
                    println!("puller: {pkh:?}, amount: {amount:?} ADA, period: {period:?} ms, next pull: {next_pull:?} ms");
                }
            })
        }
    }
    Ok(())
}

// Create dialogue for filling out the puller endpoint
async fn add_puller_impl() -> Result<()> {
    let checking_account_nft: String = dialoguer::Input::new()
        .with_prompt("Checking account NFT")
        .interact_text()?;
    let puller_pkh_string: String = dialoguer::Input::new()
        .with_prompt("Puller pub key hash")
        .interact_text()?;
    let puller_pkh_bytes = hex::decode(puller_pkh_string)?;
    let puller = PubKeyHash::new(&puller_pkh_bytes);
    let amount_ada: u64 = dialoguer::Input::new()
        .with_prompt("Amount of ADA to pull")
        .interact_text()?;
    let amount_lovelace = amount_ada * 1_000_000;
    let period: i64 = dialoguer::Input::new()
        .with_prompt("Period in milliseconds")
        .interact_text()?;
    let next_pull: i64 = dialoguer::Input::new()
        .with_prompt("Next pull in milliseconds")
        .interact_text()?;
    let endpoint = CheckingAccountEndpoints::AddPuller {
        checking_account_nft,
        puller,
        amount_lovelace,
        period,
        next_pull,
    };
    hit_endpoint(endpoint).await
}
