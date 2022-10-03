use always::logic::{
    AlwaysSucceedsEndpoints, AlwaysSucceedsLogic, AlwaysSucceedsLookupResponses,
    AlwaysSucceedsLookups,
};
use blockfrost_http_client::load_key_from_file;
use clap::Parser;
use naumachia::output::OutputId;
use naumachia::trireme_ledger_client::get_trireme_ledger_client_from_file;
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
    /// Lock amount at script address
    Lock { amount: f64 },
    /// Claim locked Output at script address
    Claim { tx_hash: String, index: u64 },
    /// List all outputs locked at script address
    List { count: usize },
}

const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
const CONFIG_FILE: &str = ".blockfrost.toml";

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = AlwaysSucceedsLogic;

    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let backend = Backend::new(ledger_client);

    let contract = SmartContract::new(&logic, &backend);

    match args.action {
        ActionParams::Lock { amount } => contract
            .hit_endpoint(AlwaysSucceedsEndpoints::Lock {
                amount: (amount * 1_000_000.) as u64,
            })
            .await
            .unwrap(),
        ActionParams::Claim { tx_hash, index } => {
            let output_id = OutputId::new(tx_hash, index);
            let endpoint = AlwaysSucceedsEndpoints::Claim { output_id };
            contract.hit_endpoint(endpoint).await.unwrap()
        }
        ActionParams::List { count } => {
            let res = contract
                .lookup(AlwaysSucceedsLookups::ListActiveContracts { count })
                .await
                .unwrap();
            match res {
                AlwaysSucceedsLookupResponses::ActiveContracts(outputs) => {
                    println!("Active contracts:");
                    for output in outputs {
                        println!("-------------------------------------");
                        println!("{:?}", output.id());
                        println!("{:?}", output.values());
                        println!("{:?}", output.datum());
                    }
                }
            }
        }
    }
}

async fn get_cml_client() -> CMLLedgerCLient<BlockFrostLedger, KeyManager, (), ()> {
    let api_key = load_key_from_file(CONFIG_FILE).unwrap();
    let ledger = BlockFrostLedger::new(TEST_URL, &api_key);
    let keys = KeyManager::new(CONFIG_FILE.to_string(), TESTNET);
    CMLLedgerCLient::<_, _, (), ()>::new(ledger, keys, TESTNET)
}
