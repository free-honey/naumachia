use always::logic::{AlwaysSucceedsEndpoints, AlwaysSucceedsLogic};
use clap::Parser;
use naumachia::{
    backend::Backend,
    ledger_client::cml_client::{
        blockfrost_ledger::BlockFrostLedger,
        key_manager::{KeyManager, TESTNET},
        CMLLedgerCLient,
    },
    smart_contract::{SmartContract, SmartContractTrait},
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    /// Create escrow contract for amount that only receiver can retrieve
    Lock { amount: u64 },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = AlwaysSucceedsLogic;

    let url = "";
    let key = "";
    let ledger = BlockFrostLedger::new(url, key);
    let keys = KeyManager::new("".to_string(), TESTNET);
    let ledger_client = CMLLedgerCLient::new(ledger, keys, TESTNET);
    let backend = Backend::new(ledger_client);

    let contract = SmartContract::new(&logic, &backend);

    match args.action {
        ActionParams::Lock { amount } => {
            contract.hit_endpoint(AlwaysSucceedsEndpoints::Lock { amount })
        }
    }
    .await
    .unwrap();
}
