use anyhow::Result;
use clap::Parser;
use dialoguer::Input;
use naumachia::trireme_ledger_client::{
    blockfrost_ledger::{write_blockfrost_api_key_to_file, BlockfrostApiKey},
    path_to_trireme_config_dir,
    raw_secret_phrase::{write_secret_phrase_to_file, SecretPhrase},
    write_toml_struct_to_file, KeySource, LedgerSource, Network, TriremeConfig,
};
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: ActionParams,
}

#[derive(clap::Subcommand, Debug)]
enum ActionParams {
    /// Initialize Trireme configuration 🚣
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        ActionParams::Init => init_impl().await?,
    }
    Ok(())
}

async fn init_impl() -> Result<()> {
    println!("🌊 Welcome to Trireme 👁");
    println!();
    println!("⚠️  Trireme is under developement! Please do not use your HODL keys!");
    println!("⚠️  Only use keys you are willing to loose funds from, preferably ");
    println!("⚠️  only with funds on testnet!");
    println!("⚠️  Trireme only works on Testnet currently!");
    println!("⚠️  Keys will be stored in plaintext files on your computer!");
    println!();
    let blockfrost_api_key_path = handle_blockfrost_api_key().await?;
    let secret_phrase_path = handle_secret_phrase().await?;
    write_trireme_config(blockfrost_api_key_path, secret_phrase_path).await?;
    println!();
    println!();
    println!("Initialized successfully!");
    println!();
    println!("🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊");
    Ok(())
}

async fn handle_secret_phrase() -> Result<PathBuf> {
    let secret_phrase: String = Input::new()
        .with_prompt("⚠️  Insert testnet secret phrase ⚠️  ")
        .interact_text()?;
    let file_path = write_secret_phrase(&secret_phrase).await?;
    Ok(file_path)
}

const RAW_PHRASE_FILE: &str = "secret_phrase.toml";

async fn write_secret_phrase(phrase: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(RAW_PHRASE_FILE);
    let phrase_struct = SecretPhrase::from_str(&phrase)?;
    // write_secret_phrase_to_file(&file_path, &phrase_struct).await?;
    write_toml_struct_to_file(&file_path, &phrase_struct).await?;
    Ok(file_path)
}

async fn handle_blockfrost_api_key() -> Result<PathBuf> {
    let api_key: String = Input::new()
        .with_prompt("Insert blockfrost testnet api key")
        .interact_text()?;
    let file_path = write_blockfrost_api_key(&api_key).await?;
    Ok(file_path)
}

const BLOCKFROST_API_KEY_FILE: &str = "blockfrost_api_key.toml";

async fn write_blockfrost_api_key(api_key: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(BLOCKFROST_API_KEY_FILE);
    let api_key_struct = BlockfrostApiKey::from_str(&api_key)?;
    // write_blockfrost_api_key_to_file(&file_path, &api_key_struct).await?;
    write_toml_struct_to_file(&file_path, &api_key_struct).await?;
    Ok(file_path)
}

const TRIREME_CONFIG_FILE: &str = "config.toml";

async fn write_trireme_config(api_key_file: PathBuf, phrase_file: PathBuf) -> Result<()> {
    let ledger_source = LedgerSource::BlockFrost { api_key_file };
    let key_source = KeySource::RawSecretPhrase { phrase_file };
    let network = Network::Testnet;
    let trireme_config = TriremeConfig::new(ledger_source, key_source, network);
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(TRIREME_CONFIG_FILE);
    write_toml_struct_to_file(&file_path, &trireme_config).await?;
    Ok(())
}
