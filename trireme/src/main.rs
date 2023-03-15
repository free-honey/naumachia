use crate::environment::remove_env_impl;
use crate::{
    balance::ada_balance_impl,
    balance::balance_impl,
    environment::new_env_impl,
    environment::{env_impl, switch_env_impl},
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
    }
    Ok(())
}
