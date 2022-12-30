use clap::Parser;
use naumachia::{
    backend::Backend,
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
    Lock { amount: f64 },
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
    let backend = Backend::new(ledger_client);
    let contract = SmartContract::new(&logic, &backend);

    println!("hello world");
    match args.action {
        ActionParams::Lock { amount } => contract
            .hit_endpoint(TimeLockedEndpoints::Lock {
                amount: (amount * 1_000_000.) as u64,
                // TODO
                timestamp: 0,
            })
            .await
            .unwrap(),
        ActionParams::Claim { tx_hash, index } => {
            let output_id = OutputId::new(tx_hash, index);
            let endpoint = TimeLockedEndpoints::Claim { output_id };
            contract.hit_endpoint(endpoint).await.unwrap()
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
                        println!("{:?}", output.id());
                        println!("{:?}", output.values());
                        println!("{:?}", output.datum());
                    }
                }
            }
        }
    }
}
