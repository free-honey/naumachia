use always_succeeds_contract::logic::{
    AlwaysSucceedsEndpoints, AlwaysSucceedsLogic, AlwaysSucceedsLookupResponses,
    AlwaysSucceedsLookups,
};
use clap::Parser;
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
            let tx_hash_bytes = hex::decode(tx_hash).unwrap();
            let output_id = OutputId::new(tx_hash_bytes, index);
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
