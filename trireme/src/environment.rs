use anyhow::Result;
use dialoguer::{Input, Select};
use naumachia::{
    error::Error,
    ledger_client::test_ledger_client::local_persisted_storage::LocalPersistedStorage,
    trireme_ledger_client::{
        cml_client::blockfrost_ledger::BlockfrostApiKey, get_trireme_config_from_file,
        path_to_client_config_file, path_to_trireme_config_dir, path_to_trireme_config_file,
        raw_secret_phrase::SecretPhrase, write_toml_struct_to_file, ClientConfig, KeySource,
        LedgerSource, Network, TriremeConfig,
    },
    Address,
};
use std::{path::PathBuf, str::FromStr};
use tokio::fs;

pub enum EnvironmentType {
    Blockfrost,
    LocalMocked,
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            EnvironmentType::Blockfrost => "Blockfrost API".to_string(),
            EnvironmentType::LocalMocked => "Local Mocked".to_string(),
        }
    }
}

pub async fn new_env_impl() -> Result<()> {
    println!();
    println!("🌊 Welcome to Trireme 👁");
    println!();
    print_safety_warning();
    let name: String = Input::new()
        .with_prompt("Please name your environment")
        .interact_text()?;
    let sub_dir = name.clone();

    let trireme_config = match get_trireme_config_from_file().await? {
        Some(mut config) => {
            config.set_new_env(&name)?;
            config
        }
        None => TriremeConfig::new(&name),
    };

    let items = vec![EnvironmentType::Blockfrost, EnvironmentType::LocalMocked];
    let item_index = Select::new()
        .with_prompt("What kind of environment?")
        .items(&items)
        .interact()?;
    let env_variant = items
        .get(item_index)
        .expect("Should always be a valid index");

    match env_variant {
        EnvironmentType::Blockfrost => {
            let api_key: String = Input::new()
                .with_prompt("Insert blockfrost testnet api key")
                .interact_text()?;
            let secret_phrase: String = Input::new()
                .with_prompt("⚠️  Insert testnet secret phrase ⚠️  ")
                .interact_text()?;
            let blockfrost_api_key_path = write_blockfrost_api_key(&api_key, &sub_dir).await?;
            let secret_phrase_path = write_secret_phrase(&secret_phrase, &sub_dir).await?;
            write_cml_client_config(&name, &sub_dir, blockfrost_api_key_path, secret_phrase_path)
                .await?;
        }
        EnvironmentType::LocalMocked => {
            // TODO: Add other keys
            let alice = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr")?;
            let start_balance = 100_000_000_000; // Lovelace
            let dir = path_to_client_config_file(&sub_dir)?;
            let parent_dir = dir.parent().ok_or(Error::Trireme(
                "Could not find parent directory for config".to_string(),
            ))?;
            fs::create_dir_all(&parent_dir).await?;
            let _ =
                LocalPersistedStorage::<PathBuf, ()>::init(parent_dir.into(), alice, start_balance);
            let client_config = ClientConfig::new_test(&name, &(parent_dir.into()));
            write_toml_struct_to_file(&dir, &client_config).await?;
        }
    }

    write_trireme_config(&trireme_config).await?;
    println!();
    println!();
    println!("Initialized successfully!");
    println!();
    println!("🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊");
    Ok(())
}

pub async fn switch_env_impl() -> Result<()> {
    match get_trireme_config_from_file().await? {
        Some(mut config) => {
            let items = config.envs();
            let index = Select::new()
                .with_prompt("To which environment?")
                .items(&items)
                .interact()?;
            let name = items.get(index).expect("should always be a valid index");
            config.switch_env(&name)?;
            write_trireme_config(&config).await?;
            println!("Switched environment to: {}", &name);
            Ok(())
        }
        None => Err(Error::Trireme("Environment doesn't exist".to_string()).into()),
    }
}

pub async fn remove_env_impl() -> Result<()> {
    match get_trireme_config_from_file().await? {
        Some(mut config) => {
            let items = config.envs();
            let index = Select::new()
                .with_prompt("Delete which environment?")
                .items(&items)
                .interact()?;
            let name = items.get(index).expect("should always be a valid index");
            let confirmation_name: String = Input::new()
                .with_prompt("Type in name of env to confirm:")
                .interact_text()?;
            if &confirmation_name == name {
                config.remove_env(&name)?;
                write_trireme_config(&config).await?;
                delete_directory(&name).await?;
                println!("🌀 Removed env: {}", &name);
            } else {
                println!("Confirmation name doesn't match. Deletion Aborted ⚓")
            }

            Ok(())
        }
        None => Err(Error::Trireme("Environment doesn't exist".to_string()).into()),
    }
}

pub async fn env_impl() -> Result<()> {
    const ERROR_MSG: &str = "No Environment Set";
    if let Some(env) = get_trireme_config_from_file()
        .await?
        .and_then(|config| config.current_env())
    {
        println!("Current Environment:");
        println!("{}", env);
    } else {
        println!("No environment set");
    }
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

async fn write_trireme_config(trireme_config: &TriremeConfig) -> Result<()> {
    let file_path = path_to_trireme_config_file()?;
    write_toml_struct_to_file(&file_path, trireme_config).await?;
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

async fn delete_directory(sub_dir: &str) -> Result<()> {
    let file_path = path_to_client_config_file(sub_dir)?;
    let parent_dir = file_path.parent().ok_or(Error::Trireme(
        "Could not find parent directory for config".to_string(),
    ))?;
    fs::remove_dir_all(parent_dir).await?;
    Ok(())
}