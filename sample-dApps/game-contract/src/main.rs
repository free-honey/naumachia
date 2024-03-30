use clap::Parser;
use game_contract::logic::{
    GameEndpoints,
    GameLogic,
    GameLookupResponses,
    GameLookups,
};
use naumachia::{
    output::OutputId,
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
    /// Lock amount at script address
    Lock { amount: f64, secret: String },
    /// Claim locked Output at script address
    Guess {
        tx_hash: String,
        index: u64,
        guess: String,
    },
    /// List all outputs locked at script address
    List { count: usize },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = GameLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let contract = SmartContract::new(logic, ledger_client);

    match args.action {
        ActionParams::Lock { amount, secret } => {
            let tx_id = contract
                .hit_endpoint(GameEndpoints::Lock {
                    amount: (amount * 1_000_000.) as u64,
                    secret,
                })
                .await
                .unwrap();
            println!("tx: {:?}", tx_id);
        }
        ActionParams::Guess {
            tx_hash,
            index,
            guess,
        } => {
            let tx_hash_bytes = hex::decode(tx_hash).unwrap();
            let output_id = OutputId::new(tx_hash_bytes, index);
            let endpoint = GameEndpoints::Guess { output_id, guess };
            let tx_id = contract.hit_endpoint(endpoint).await.unwrap();
            println!("tx: {:?}", tx_id);
        }
        ActionParams::List { count } => {
            let res = contract
                .lookup(GameLookups::ListActiveContracts { count })
                .await
                .unwrap();
            match res {
                GameLookupResponses::ActiveContracts(outputs) => {
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
