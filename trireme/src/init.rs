use anyhow::Result;
use dialoguer::Input;
use naumachia::trireme_ledger_client::cml_client::blockfrost_ledger::BlockfrostApiKey;
use naumachia::trireme_ledger_client::{path_to_client_config_file, TriremeConfig};
use naumachia::{
    trireme_ledger_client::raw_secret_phrase::SecretPhrase,
    trireme_ledger_client::{
        path_to_trireme_config_dir, path_to_trireme_config_file, write_toml_struct_to_file,
        ClientConfig, KeySource, LedgerSource, Network,
    },
};
use std::collections::HashMap;
use std::{path::PathBuf, str::FromStr};
use uuid::Uuid;

pub async fn init_impl() -> Result<()> {
    println!();
    println!("🌊 Welcome to Trireme 👁");
    println!();
    print_safety_warning();
    let name: String = Input::new()
        .with_prompt("Please name your environment")
        .interact_text()?;
    let sub_dir = Uuid::new_v4().to_string();
    let api_key: String = Input::new()
        .with_prompt("Insert blockfrost testnet api key")
        .interact_text()?;
    let secret_phrase: String = Input::new()
        .with_prompt("⚠️  Insert testnet secret phrase ⚠️  ")
        .interact_text()?;
    let blockfrost_api_key_path = write_blockfrost_api_key(&api_key, &sub_dir).await?;
    let secret_phrase_path = write_secret_phrase(&secret_phrase, &sub_dir).await?;
    let current_env = name.clone();
    let mut envs = HashMap::new();
    envs.insert(name.clone(), sub_dir.clone());
    write_trireme_config(&current_env, envs).await?;
    write_cml_client_config(&name, &sub_dir, blockfrost_api_key_path, secret_phrase_path).await?;
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

async fn write_secret_phrase(phrase: &str, sub_dir: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(sub_dir);
    file_path.push(RAW_PHRASE_FILE);
    let phrase_struct = SecretPhrase::from_str(&phrase)?;
    write_toml_struct_to_file(&file_path, &phrase_struct).await?;
    Ok(file_path)
}

const BLOCKFROST_API_KEY_FILE: &str = "blockfrost_api_key.toml";

async fn write_blockfrost_api_key(api_key: &str, sub_dir: &str) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(sub_dir);
    file_path.push(BLOCKFROST_API_KEY_FILE);
    let api_key_struct = BlockfrostApiKey::from_str(&api_key)?;
    write_toml_struct_to_file(&file_path, &api_key_struct).await?;
    Ok(file_path)
}

async fn write_trireme_config(current_env: &str, envs: HashMap<String, String>) -> Result<()> {
    let trireme_config = TriremeConfig::new(current_env, envs);
    let file_path = path_to_trireme_config_file()?;
    write_toml_struct_to_file(&file_path, &trireme_config).await?;
    Ok(())
}

async fn write_cml_client_config(
    name: &str,
    sub_dir: &str,
    api_key_file: PathBuf,
    phrase_file: PathBuf,
) -> Result<()> {
    let ledger_source = LedgerSource::BlockFrost { api_key_file };
    let key_source = KeySource::RawSecretPhrase { phrase_file };
    let network = Network::Preprod;
    let client_config = ClientConfig::new_cml(name, ledger_source, key_source, network);
    let file_path = path_to_client_config_file(sub_dir)?;
    write_toml_struct_to_file(&file_path, &client_config).await?;
    Ok(())
}
