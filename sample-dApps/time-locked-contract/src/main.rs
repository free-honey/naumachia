use clap::Parser;
use naumachia::{
    output::OutputId,
    smart_contract::{SmartContract, SmartContractTrait},
    trireme_ledger_client::get_trireme_ledger_client_from_file,
};
use time_locked_contract::logic::{
    TimeLockedEndpoints, TimeLockedLogic, TimeLockedLookupResponses, TimeLockedLookups,
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
    Lock { amount: f64, after_secs: i64 },
    /// Claim locked Output at script address
    Claim { tx_hash: String, index: u64 },
    /// List all outputs locked at script address
    List { count: usize },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let logic = TimeLockedLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await.unwrap();
    let contract = SmartContract::new(logic, ledger_client);

    match args.action {
        ActionParams::Lock { amount, after_secs } => {
            let tx_id = contract
                .hit_endpoint(TimeLockedEndpoints::Lock {
                    amount: (amount * 1_000_000.) as u64,
                    after_secs,
                })
                .await
                .unwrap();
            println!("TxId: {:?}", tx_id);
        },
        ActionParams::Claim { tx_hash, index } => {
            let tx_hash_bytes = hex::decode(tx_hash).unwrap();
            let output_id = OutputId::new(tx_hash_bytes, index);
            let endpoint = TimeLockedEndpoints::Claim { output_id };
            match contract.hit_endpoint(endpoint).await {
                Ok(tx_id) => println!("Claimed output :) with tx_id: {:?}", tx_id),
                Err(e) => println!("Error claiming output: {:?}", e),
            }
        }
        ActionParams::List { count } => {
            let res = contract
                .lookup(TimeLockedLookups::ListActiveContracts { count })
                .await
                .unwrap();
            match res {
                TimeLockedLookupResponses::ActiveContracts(outputs) => {
                    println!("Active contracts:");
                    for output in outputs {
                        println!("-------------------------------------");
                        println!(
                            "hash: {:?}, index: {:?}",
                            hex::encode(output.id().tx_hash()),
                            output.id().index()
                        );
                        println!("{:?}", output.values());
                        println!("{:?}", output.datum());
                    }
                }
            }
        }
    }
}
