use crate::Error;
use anyhow::Result;
use dialoguer::{Input, Password as InputPassword, Select};
use hex;
use naumachia::scripts::context::pub_key_hash_from_address_if_available;
use naumachia::trireme_ledger_client::get_current_client_config_from_file;
use naumachia::trireme_ledger_client::terminal_password_phrase::{
    encrypt_phrase, normalize_password,
};
use naumachia::{
    ledger_client::{
        test_ledger_client::local_persisted_storage::LocalPersistedStorage, LedgerClient,
    },
    trireme_ledger_client::{
        cml_client::blockfrost_ledger::BlockfrostApiKey, get_trireme_config_from_file,
        get_trireme_ledger_client_from_file, path_to_client_config_file,
        path_to_trireme_config_dir, path_to_trireme_config_file, read_toml_struct_from_file,
        write_toml_struct_to_file, ClientConfig, ClientVariant, KeySource, LedgerSource, Network,
        TriremeConfig, TriremeLedgerClient,
    },
    Address,
};
use rand::Rng;
use std::{path::PathBuf, str::FromStr};
use tokio::fs;

#[derive(Clone, Copy)]
pub enum EnvironmentType {
    Real,
    Mocked,
}

impl ToString for EnvironmentType {
    fn to_string(&self) -> String {
        match self {
            EnvironmentType::Real => "Real Chain".to_string(),
            EnvironmentType::Mocked => "Local Mocked".to_string(),
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

    let trireme_config = match get_trireme_config_from_file().await? {
        Some(mut config) => {
            config.set_new_env(&name)?;
            config
        }
        None => TriremeConfig::new(&name),
    };

    match get_env_type()? {
        EnvironmentType::Real => setup_password_protected_blockfrost_env(&name).await?,
        EnvironmentType::Mocked => setup_local_mocked_env(&name).await?,
    }

    write_trireme_config(&trireme_config).await?;
    println!();
    println!();
    println!("Initialized successfully!");
    println!();
    println!("🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊🌊");
    Ok(())
}

fn get_env_type() -> Result<EnvironmentType> {
    let items = vec![EnvironmentType::Mocked, EnvironmentType::Real];
    let item_index = Select::new()
        .with_prompt("What kind of environment?")
        .items(&items)
        .interact()?;
    let env_type = items
        .get(item_index)
        .expect("Should always be a valid index")
        .to_owned();
    Ok(env_type)
}

async fn setup_password_protected_blockfrost_env(name: &str) -> Result<()> {
    let ledger_source = get_ledger_source(name).await?;

    let secret_phrase: String = Input::new()
        .with_prompt("⚠️  Insert testnet secret phrase ⚠️  ")
        .interact_text()?;

    let password = get_password_with_prompt("Enter password")?;
    let mut confirmed_password = get_password_with_prompt("Confirm password")?;

    while password != confirmed_password {
        println!("Try again");
        confirmed_password = get_password_with_prompt("Confirm password")?;
    }

    // TODO: Is `rand` good enough for salt?
    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let normalized_password = normalize_password(&password, &salt)?;

    let encryption_nonce = rand::thread_rng().gen::<[u8; 12]>();

    let secret_phrase_path = write_secret_phrase_with_password(
        &secret_phrase,
        name,
        &normalized_password,
        &encryption_nonce,
    )
    .await?;
    // TODO: Do a prompt or derive network from api key
    let network = Network::Preprod;

    write_cml_client_config_with_password_protection(
        &name,
        &name,
        ledger_source,
        secret_phrase_path,
        salt.to_vec(),
        encryption_nonce,
        network,
    )
    .await?;
    Ok(())
}

#[derive(Debug)]
enum LedgerTypes {
    BlockFrost,
    OgmiosAndScrolls,
}

impl ToString for LedgerTypes {
    fn to_string(&self) -> String {
        match self {
            LedgerTypes::BlockFrost => "Blockfrost API".to_string(),
            LedgerTypes::OgmiosAndScrolls => "Ogmios and Scrolls".to_string(),
        }
    }
}

async fn get_ledger_source(env_name: &str) -> Result<LedgerSource> {
    let items = vec![LedgerTypes::BlockFrost, LedgerTypes::OgmiosAndScrolls];
    let item_index = Select::new()
        .with_prompt("What is your ledger data provider?")
        .items(&items)
        .interact()?;
    let ledger_type = items
        .get(item_index)
        .expect("Should always be a valid index")
        .to_owned();

    match ledger_type {
        LedgerTypes::BlockFrost => setup_blockfrost_ledger(env_name).await,
        LedgerTypes::OgmiosAndScrolls => setup_ogmios_and_scrolls_ledger(),
    }
}

async fn setup_blockfrost_ledger(env_name: &str) -> Result<LedgerSource> {
    let api_key: String = Input::new()
        .with_prompt("Insert blockfrost testnet api key")
        .interact_text()?;
    let blockfrost_api_key_path = write_blockfrost_api_key(&api_key, env_name).await?;
    let ledger_source = LedgerSource::BlockFrost {
        api_key_file: blockfrost_api_key_path,
    };
    Ok(ledger_source)
}

fn setup_ogmios_and_scrolls_ledger() -> Result<LedgerSource> {
    let scrolls_ip: String = Input::new()
        .with_prompt("Ip address of Scrolls Redis server")
        .default("127.0.0.1".to_string())
        .interact_text()?;
    let scrolls_port: String = Input::new()
        .with_prompt("Port for Scrolls Redis server")
        .default("6379".to_string())
        .interact_text()?;
    let ogmios_ip: String = Input::new()
        .with_prompt("Ip address of Ogmios")
        .default("127.0.0.1".to_string())
        .interact_text()?;
    let ogmios_port: String = Input::new()
        .with_prompt("Port for Ogmios")
        .default("1337".to_string())
        .interact_text()?;
    let ledger_source = LedgerSource::OgmiosAndScrolls {
        scrolls_ip,
        scrolls_port,
        ogmios_ip,
        ogmios_port,
    };
    Ok(ledger_source)
}

fn get_password_with_prompt(prompt: &str) -> Result<String> {
    let password = InputPassword::new().with_prompt(prompt).interact()?;
    Ok(password)
}

async fn setup_local_mocked_env(name: &str) -> Result<()> {
    let block_length: i64 = Input::new()
        .with_prompt("What is the block length in secs?")
        .default(20)
        .interact_text()?;

    let alice_name = "Alice";
    let alice_address = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr")?;
    let bob_name = "Bob";
    let bob_address = Address::from_bech32("addr_test1qzulfkd06qm7t2nwe44nnuxh57k4h3p8zdrqukrjcekwn3kcra4ulhfn3g7j9gmnvmefjwzfsd55fq5ndecwlhgcw4zq07drdr")?;
    let charlotte_name = "Charlotte";
    let charlotte_address = Address::from_bech32("addr_test1qryc5tck5kqqs3arcqnl4lplvw5yg2ujsdnhx5eawn9lyzzvpmpraw365fayhrtpzpl4nulq6f9hhdkh4cdyh0tgnjxsg03qnh")?;
    let dick_name = "Dick";
    let dick_address = Address::from_bech32("addr_test1qr25qu9uu2putyngq38p04suc7w4lsgq5ylvt5q8hf3d9jh8gqwn858xkeuq7dlg5zycefeztfps6dmh62zpvac5wqxqvtgh4x")?;

    let start_balance = 100_000_000_000; // Lovelace
    let dir = path_to_client_config_file(name)?;
    let parent_dir = dir.parent().ok_or(Error::CLI(
        "Could not find parent directory for config".to_string(),
    ))?;
    fs::create_dir_all(&parent_dir).await?;

    let starting_time = 0;
    let storage = LocalPersistedStorage::<PathBuf, ()>::init(
        parent_dir.into(),
        alice_name,
        &alice_address,
        start_balance,
        starting_time,
        block_length,
    );
    storage.add_new_signer(bob_name, &bob_address, start_balance);
    storage.add_new_signer(charlotte_name, &charlotte_address, start_balance);
    storage.add_new_signer(dick_name, &dick_address, start_balance);
    let client_config = ClientConfig::new_test(&name, &(parent_dir.into()));
    write_toml_struct_to_file(&dir, &client_config).await?;
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
        None => Err(Error::CLI("Environment doesn't exist".to_string()).into()),
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
                .with_prompt("Type in name of env to confirm")
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
        None => Err(Error::CLI("Environment doesn't exist".to_string()).into()),
    }
}

pub async fn env_impl() -> Result<()> {
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

pub async fn active_signer_impl() -> Result<()> {
    let sub_dir = get_trireme_config_from_file()
        .await?
        .and_then(|config| config.current_env())
        .unwrap(); // TODO
    let dir = path_to_client_config_file(&sub_dir)?;
    let config = read_toml_struct_from_file::<ClientConfig>(&dir)
        .await?
        .unwrap(); // TODO
    match config.variant() {
        ClientVariant::Test(inner) => {
            let path = inner.data_path();
            let storage = LocalPersistedStorage::<PathBuf, ()>::load(path);
            let signer = storage.active_signer_name();
            println!("Active signer: {}", signer);
        }
        _ => {
            unimplemented!("Only the mock supports multiple signers");
        }
    }
    Ok(())
}

pub async fn get_address_impl() -> Result<()> {
    let ledger_client: TriremeLedgerClient<(), ()> = get_trireme_ledger_client_from_file().await?;
    let address = ledger_client.signer_base_address().await?;
    let address_string = address.to_bech32()?;
    println!("Address: {address_string}");
    Ok(())
}

pub async fn get_pubkey_hash_impl() -> Result<()> {
    let ledger_client: TriremeLedgerClient<(), ()> = get_trireme_ledger_client_from_file().await?;
    let address = ledger_client.signer_base_address().await?;
    let pubkey_hash = pub_key_hash_from_address_if_available(&address).ok_or(Error::CLI(
        "Could not derive Pubkey Hash from Address".to_string(),
    ))?;
    let pubkey_hash_string = hex::encode(pubkey_hash.bytes());
    println!("Pubkey Hash: {pubkey_hash_string}");
    Ok(())
}

pub async fn switch_signer_impl() -> Result<()> {
    let sub_dir = get_trireme_config_from_file()
        .await?
        .and_then(|config| config.current_env())
        .unwrap();
    let dir = path_to_client_config_file(&sub_dir)?;
    let config = read_toml_struct_from_file::<ClientConfig>(&dir)
        .await?
        .unwrap(); // TODO
    match config.variant() {
        ClientVariant::Test(inner) => {
            let path = inner.data_path();
            let storage = LocalPersistedStorage::<PathBuf, ()>::load(path);
            let items = storage.get_signers();
            let choice = Select::new()
                .with_prompt("To which signer?")
                .items(&items)
                .interact()?;
            let name = items.get(choice).expect("should always be a valid index");
            storage.switch_signer(&name);
            println!("Switched signer to: {}", &name);
        }
        _ => {
            unimplemented!("Only the mock supports adding signers");
        }
    }
    Ok(())
}

pub async fn current_time_impl() -> Result<()> {
    let maybe_config = get_current_client_config_from_file().await?;
    if let Some(config) = maybe_config {
        match config.variant() {
            ClientVariant::CML(_) => {
                let current_system_time = std::time::SystemTime::now();
                let time_from_unix_epoch = current_system_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards");
                println!(
                    "System time from UNIX epoch: {:?} secs",
                    time_from_unix_epoch.as_secs()
                );
            }
            ClientVariant::Test(_) => {
                let ledger_client: TriremeLedgerClient<(), ()> =
                    get_trireme_ledger_client_from_file().await?;
                let current_time = ledger_client.current_time().await?;
                println!(
                    "Current time according to your environment: {:?} secs",
                    current_time
                );
            }
        }
    }
    Ok(())
}

pub async fn last_block_time_impl() -> Result<()> {
    let ledger_client: TriremeLedgerClient<(), ()> = get_trireme_ledger_client_from_file().await?;
    let last_block_time = ledger_client.last_block_time_secs().await?;
    println!("Last block time: {}", last_block_time);
    Ok(())
}

pub async fn advance_blocks(count: i64) -> Result<()> {
    let ledger_client: TriremeLedgerClient<(), ()> = get_trireme_ledger_client_from_file().await?;
    ledger_client.advance_blocks(count).await?;
    println!("Advancing blocks by: {}", count);
    let block_time = ledger_client.current_time().await?;
    println!("New block time: {}", block_time);
    Ok(())
}

const RAW_PHRASE_FILE: &str = "secret_phrase.toml";

async fn write_secret_phrase_with_password(
    phrase: &str,
    sub_dir: &str,
    password: &[u8; 32],
    encryption_nonce: &[u8; 12],
) -> Result<PathBuf> {
    let mut file_path = path_to_trireme_config_dir()?;
    file_path.push(sub_dir);
    file_path.push(RAW_PHRASE_FILE);
    let phrase_struct = encrypt_phrase(phrase, password, encryption_nonce);
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

async fn write_cml_client_config_with_password_protection(
    name: &str,
    sub_dir: &str,
    ledger_source: LedgerSource,
    phrase_file: PathBuf,
    password_salt: Vec<u8>,
    encrpytion_nonce: [u8; 12],
    network: Network,
) -> Result<()> {
    let key_source = KeySource::TerminalPasswordUpfrontSecretPhrase {
        phrase_file,
        password_salt,
        encrpytion_nonce,
    };
    let client_config = ClientConfig::new_cml(name, ledger_source, key_source, network);
    let file_path = path_to_client_config_file(sub_dir)?;
    write_toml_struct_to_file(&file_path, &client_config).await?;
    Ok(())
}

async fn delete_directory(sub_dir: &str) -> Result<()> {
    let file_path = path_to_client_config_file(sub_dir)?;
    let parent_dir = file_path.parent().ok_or(Error::CLI(
        "Could not find parent directory for config".to_string(),
    ))?;
    fs::remove_dir_all(parent_dir).await?;
    Ok(())
}
