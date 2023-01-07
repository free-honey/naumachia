use anyhow::Result;
use dialoguer::Input;
use naumachia::trireme_ledger_client::cml_client::blockfrost_ledger::BlockfrostApiKey;
use naumachia::{
    trireme_ledger_client::raw_secret_phrase::SecretPhrase,
    trireme_ledger_client::{
        path_to_trireme_config_dir, path_to_trireme_config_file, write_toml_struct_to_file,
        KeySource, LedgerSource, Network, TriremeConfig,
    },
};
use std::{path::PathBuf, str::FromStr};

pub async fn init_impl() -> Result<()> {
    println!();
    println!("🌊 Welcome to Trireme 👁");
    println!();
    print_safety_warning();
    let api_key: String = Input::new()
        .with_prompt("Insert blockfrost testnet api key")
        .interact_text()?;
    let secret_phrase: String = Input::new()
        .with_prompt("⚠️  Insert testnet secret phrase ⚠️  ")
        .interact_text()?;
    let blockfrost_api_key_path = write_blockfrost_api_key(&api_key).await?;
    let secret_phrase_path = write_secret_phrase(&secret_phrase).await?;
    write_trireme_config(blockfrost_api_key_path, secret_phrase_path).await?;
    println!();
    println!();
    println!("Initialized successfully!");
    println!();
    println!("🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊");
    Ok(())
}

fn print_safety_warning() {
    println!("⚠️  Trireme is under developement! Please do not use your HODL keys!");
    println!("⚠️  Only use keys you are willing to loose funds from, preferably ");
    println!("⚠️  only with funds on testnet!");
    println!("⚠️  Trireme only works on Testnet currently!");
    println!("⚠️  Keys will be stored in plaintext files on your computer!");
    println!();
}

const RAW_PHRASE_FILE: &str = "secret_phrase.toml";

async fn write_secret_phrase(phrase: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(RAW_PHRASE_FILE);
    let phrase_struct = SecretPhrase::from_str(&phrase)?;
    write_toml_struct_to_file(&file_path, &phrase_struct).await?;
    Ok(file_path)
}

const BLOCKFROST_API_KEY_FILE: &str = "blockfrost_api_key.toml";

async fn write_blockfrost_api_key(api_key: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(BLOCKFROST_API_KEY_FILE);
    let api_key_struct = BlockfrostApiKey::from_str(&api_key)?;
    write_toml_struct_to_file(&file_path, &api_key_struct).await?;
    Ok(file_path)
}

async fn write_trireme_config(api_key_file: PathBuf, phrase_file: PathBuf) -> Result<()> {
    let ledger_source = LedgerSource::BlockFrost { api_key_file };
    let key_source = KeySource::RawSecretPhrase { phrase_file };
    let network = Network::Preprod;
    let trireme_config = TriremeConfig::new(ledger_source, key_source, network);
    let file_path = path_to_trireme_config_file()?;
    write_toml_struct_to_file(&file_path, &trireme_config).await?;
    Ok(())
}
