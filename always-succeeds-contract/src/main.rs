use always::logic::{AlwaysSucceedsEndpoints, AlwaysSucceedsLogic};
use blockfrost_http_client::load_key_from_file;
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
    Lock { amount: f64 },
}

const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
const CONFIG_FILE: &str = ".blockfrost.toml";

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = AlwaysSucceedsLogic;

    let ledger_client = get_cml_client().await;
    let backend = Backend::new(ledger_client);

    let contract = SmartContract::new(&logic, &backend);

    match args.action {
        ActionParams::Lock { amount } => contract.hit_endpoint(AlwaysSucceedsEndpoints::Lock {
            amount: (amount * 1_000_000.) as u64,
        }),
    }
    .await
    .unwrap();
}

async fn get_cml_client() -> CMLLedgerCLient<BlockFrostLedger, KeyManager, (), ()> {
    let api_key = load_key_from_file(CONFIG_FILE).unwrap();
    let ledger = BlockFrostLedger::new(TEST_URL, &api_key);
    let keys = KeyManager::new(CONFIG_FILE.to_string(), TESTNET);
    CMLLedgerCLient::<_, _, (), ()>::new(ledger, keys, TESTNET)
}
