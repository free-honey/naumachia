use crate::environment::{
    active_signer_impl, advance_blocks, current_time_impl, get_address_impl, get_pubkey_hash_impl,
    last_block_time_impl, switch_signer_impl,
};
use crate::{
    balance::{ada_balance_impl, balance_impl},
    environment::{env_impl, new_env_impl, remove_env_impl, switch_env_impl},
    logic::{TriremeLogic, TriremeLookups, TriremeResponses},
};
use anyhow::Result;
use clap::Parser;
use thiserror::Error;

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
    /// View current env info 🌄
    Env,
    /// Create a new environment 🚣
    NewEnv,
    /// Switch Environments ⛵
    SwitchEnv,
    /// Remove Env 🌀
    RemoveEnv,
    /// Get ADA Balance ₳
    AdaBalance,
    /// Get Total Balance 💰
    Balance,
    /// Get Signer's Base Address 📬
    Address,
    /// Get Signer's Hex Encoded Public Key Hash 🗝
    PubKeyHash,
    /// Reports active signer 😊 (Mock Network Only)
    Signer,
    /// Switch to different signer 👽 (Mock Network Only)
    SwitchSigner,
    /// Get get time relative to your local environment 🕰
    Time,
    /// Get the time of the last block in seconds
    LastBlockTime,
    /// Advance time and block height by count 🧱
    AdvanceBlocks { count: u16 },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error within the CLI Logic: {0}")]
    CLI(String),
    #[error("Error with Password: {0}")]
    Password(String),
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
        ActionParams::Address => get_address_impl().await?,
        ActionParams::PubKeyHash => get_pubkey_hash_impl().await?,
        ActionParams::Signer => active_signer_impl().await?,
        ActionParams::SwitchSigner => switch_signer_impl().await?,
        ActionParams::Time => current_time_impl().await?,
        ActionParams::LastBlockTime => last_block_time_impl().await?,
        ActionParams::AdvanceBlocks { count } => advance_blocks(count as i64).await?,
    }
    Ok(())
}
