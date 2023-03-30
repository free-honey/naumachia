use crate::environment::{active_signer_impl, switch_signer_impl};
use crate::{
    balance::{ada_balance_impl, balance_impl},
    environment::{env_impl, new_env_impl, remove_env_impl, switch_env_impl},
    logic::{TriremeLogic, TriremeLookups, TriremeResponses},
};
use anyhow::Result;
use clap::Parser;

mod balance;
mod environment;
mod logic;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    /// View current env
    Env,
    /// Create a new environment 🚣
    NewEnv,
    /// Switch Environments
    SwitchEnv,
    /// Remove Env
    RemoveEnv,
    /// Get ADA Balance
    AdaBalance,
    /// Get Total Balance
    Balance,
    /// Reports active signer
    Signer,
    /// Switch to different signer (Mock Network Only)
    SwitchSigner,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        ActionParams::Env => env_impl().await?,
        ActionParams::NewEnv => new_env_impl().await?,
        ActionParams::SwitchEnv => switch_env_impl().await?,
        ActionParams::RemoveEnv => remove_env_impl().await?,
        ActionParams::AdaBalance => ada_balance_impl().await?,
        ActionParams::Balance => balance_impl().await?,
        ActionParams::Signer => active_signer_impl().await?,
        ActionParams::SwitchSigner => switch_signer_impl().await?,
    }
    Ok(())
}
