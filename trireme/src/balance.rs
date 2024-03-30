use crate::{
    TriremeLogic,
    TriremeLookups,
    TriremeResponses,
};
use anyhow::Result;
use naumachia::{
    error::Error,
    policy_id::PolicyId,
    smart_contract::{
        SmartContract,
        SmartContractTrait,
    },
    trireme_ledger_client::get_trireme_ledger_client_from_file,
};

async fn run_lookup(lookup: TriremeLookups) -> Result<TriremeResponses> {
    let logic = TriremeLogic;
    let ledger_client = get_trireme_ledger_client_from_file().await?;
    let contract = SmartContract::new(logic, ledger_client);
    let res = contract.lookup(lookup).await?;
    Ok(res)
}

fn lovelace_to_ada(lovelace: f64) -> f64 {
    lovelace / 1_000_000.0 // TODO: Panic
}

pub(crate) async fn ada_balance_impl() -> Result<()> {
    let lookup = TriremeLookups::LovelaceBalance;
    match run_lookup(lookup).await? {
        TriremeResponses::LovelaceBalance(lovelace) => {
            let ada = lovelace_to_ada(lovelace as f64);
            println!("Balance: {:?} ADA", ada);
            Ok(())
        }
        _ => Err(Error::Trireme("Failed to retrieve balance".to_string()).into()),
    }
}

pub(crate) async fn balance_impl() -> Result<()> {
    let lookup = TriremeLookups::TotalBalance;
    match run_lookup(lookup).await? {
        TriremeResponses::TotalBalance(assets) => {
            let mut ada_balance: f64 = 0.0;
            let mut native_assets = Vec::new();
            for (id, amt) in assets {
                match id {
                    PolicyId::Lovelace => {
                        ada_balance = lovelace_to_ada(amt as f64);
                    }
                    PolicyId::NativeToken(policy_id, asset_name) => {
                        let asset = (policy_id, asset_name.unwrap_or(String::new()), amt);
                        native_assets.push(asset);
                    }
                }
            }
            println!("Balances:");
            println!("{:?} ADA", ada_balance);
            for (id, name, amt) in native_assets {
                println!("{:?} {} ({})", amt, name, id);
            }
            Ok(())
        }
        _ => Err(Error::Trireme("Failed to retrieve balances".to_string()).into()),
    }
}
