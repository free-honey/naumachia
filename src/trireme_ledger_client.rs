use crate::{
    error::*,
    ledger_client::{
        test_ledger_client::local_persisted_storage::LocalPersistedStorage,
        test_ledger_client::TestLedgerClient, LedgerClient, LedgerClientError, LedgerClientResult,
    },
    output::Output,
    scripts::plutus_validator::plutus_data::PlutusData,
    transaction::TxId,
    trireme_ledger_client::cml_client::blockfrost_ledger::BlockfrostApiKey,
    trireme_ledger_client::raw_secret_phrase::RawSecretPhraseKeys,
    UnbuiltTransaction,
};

use crate::trireme_ledger_client::cml_client::network_settings::NetworkSettings;
use crate::trireme_ledger_client::cml_client::ogmios_scrolls_ledger::OgmiosScrollsLedger;
use crate::trireme_ledger_client::cml_client::Keys;
use crate::trireme_ledger_client::terminal_password_phrase::{
    PasswordProtectedPhraseKeys, TerminalPasswordUpfront,
};
use async_trait::async_trait;
use blockfrost_http_client::{MAINNET_URL, PREPROD_NETWORK_URL};
use cardano_multiplatform_lib::address::BaseAddress;
use cardano_multiplatform_lib::crypto::PrivateKey;
use cml_client::{
    blockfrost_ledger::BlockFrostLedger, plutus_data_interop::PlutusDataInterop, CMLLedgerCLient,
};
use dirs::home_dir;
use ogmios_client::OgmiosClient;
use pallas_addresses::Address;
use scrolls_client::ScrollsClient;
use serde::{de::DeserializeOwned, ser, Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData, path::PathBuf};
use thiserror::Error;
use tokio::{fs, io::AsyncWriteExt};

/// CML CLient module
pub mod cml_client;
/// Raw secret phrase module
pub mod raw_secret_phrase;
/// Secret Phrase module
pub mod secret_phrase;
/// Terminal password upfront module
pub mod terminal_password_phrase;

/// Default Trireme config folder
pub const TRIREME_CONFIG_FOLDER: &str = ".trireme";
/// Default Trireme config file name
pub const TRIREME_CONFIG_FILE: &str = "config.toml";
/// Default Trireme client config file name
pub const CLIENT_CONFIG_FILE: &str = "config.toml";

/// Default Trireme config folder
pub fn path_to_trireme_config_dir() -> Result<PathBuf> {
    let mut dir =
        home_dir().ok_or_else(|| Error::Trireme("Could not find home directory :(".to_string()))?;
    dir.push(TRIREME_CONFIG_FOLDER);
    Ok(dir)
}

/// Path to the default Trireme config file
pub fn path_to_trireme_config_file() -> Result<PathBuf> {
    let mut dir = path_to_trireme_config_dir()?;
    dir.push(TRIREME_CONFIG_FILE);
    Ok(dir)
}

/// Path to specific Trireme client config file
pub fn path_to_client_config_file(sub_dir: &str) -> Result<PathBuf> {
    let mut dir = path_to_trireme_config_dir()?;
    dir.push(sub_dir);
    dir.push(CLIENT_CONFIG_FILE);
    Ok(dir)
}

/// Attempts to read the default config and returns the current client config from file
pub async fn get_current_client_config_from_file() -> Result<Option<ClientConfig>> {
    let trireme_config = get_trireme_config_from_file().await?;
    let current_env = trireme_config
        .ok_or(Error::Trireme(
            "Trireme not initialized (config not found)".to_string(),
        ))?
        .get_current_env_subdir()
        .ok_or(Error::Trireme("No environment initialized".to_string()))?;
    let client_config_path = path_to_client_config_file(&current_env)?;
    read_toml_struct_from_file::<ClientConfig>(&client_config_path).await
}

/// Attempts to read the default config and returns the current client proper based on current config
// TODO: PlutusDataInterop is prolly overconstraining for the Redeemer
pub async fn get_trireme_ledger_client_from_file<
    Datum: PlutusDataInterop
        + Clone
        + Eq
        + PartialEq
        + Debug
        + Hash
        + Send
        + Sync
        + Into<PlutusData>
        + TryFrom<PlutusData>,
    Redeemer: PlutusDataInterop + Clone + Eq + Debug + Hash + Send + Sync + DeserializeOwned,
>() -> Result<TriremeLedgerClient<Datum, Redeemer>> {
    if let Some(config) = get_current_client_config_from_file().await? {
        config.to_client().await
    } else {
        Err(Error::Trireme("Config could not be read".to_string()))
    }
}

/// Attempts to read the default Trireme config
pub async fn get_trireme_config_from_file() -> Result<Option<TriremeConfig>> {
    let trireme_config_path = path_to_trireme_config_file()?;
    read_toml_struct_from_file::<TriremeConfig>(&trireme_config_path).await
}

/// Types of providers for the Ledger
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum LedgerSource {
    /// Read from BlockFrost API
    BlockFrost {
        /// Path to API key file
        api_key_file: PathBuf,
    },
    /// Read from a Ogmios + Scrolls instance
    OgmiosAndScrolls {
        /// IP address of Scrolls instance
        scrolls_ip: String,
        /// Port of Scrolls instance
        scrolls_port: String,
        /// IP address of Ogmios instance
        ogmios_ip: String,
        /// Port of Ogmios instance
        ogmios_port: String,
    },
}

/// Type of key storage
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum KeySource {
    /// Read from a raw secret phrase file
    RawSecretPhrase {
        /// Path to secret phrase file
        phrase_file: PathBuf,
    },
    /// Read from a password-protected secret phrase file
    TerminalPasswordUpfrontSecretPhrase {
        /// Path to encrypted secret phrase file
        phrase_file: PathBuf,
        /// Salt used to hash the password
        password_salt: Vec<u8>,
        /// Nonce used to encrypt the secret phrase
        encrpytion_nonce: [u8; 12],
    },
}

/// The networks supported by Trireme
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum Network {
    /// "Preprod" Cardano test network
    Preprod,
    /// "Preview" Cardano test network
    Preview,
    /// Cardano Mainnet
    Mainnet,
}

impl From<Network> for u8 {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => 1,
            Network::Preprod | Network::Preview => 0,
        }
    }
}

/// Config for Trireme Ledger Client
#[derive(Deserialize, Serialize)]
pub struct TriremeConfig {
    current_env: Option<String>,
    envs: Vec<String>,
}

impl TriremeConfig {
    /// Constructor for TriremeConfig
    pub fn new(current_env: &str) -> Self {
        TriremeConfig {
            current_env: Some(current_env.to_string()),
            envs: vec![current_env.to_string()],
        }
    }

    /// Get the subdir for current environment
    pub fn get_current_env_subdir(&self) -> Option<String> {
        self.current_env.clone()
    }

    /// Add new environmnet and set it as current
    pub fn set_new_env(&mut self, new_env_name: &str) -> Result<()> {
        if self.envs.contains(&(new_env_name.to_string())) {
            Err(Error::Trireme(
                "Environment with that name already exists".to_string(),
            ))
        } else {
            self.current_env = Some(new_env_name.to_string());
            self.envs.push(new_env_name.to_string());
            Ok(())
        }
    }

    /// Switch current environment to another existing environment
    pub fn switch_env(&mut self, env_name: &str) -> Result<()> {
        if self.envs.contains(&(env_name.to_string())) {
            self.current_env = Some(env_name.to_string());
            Ok(())
        } else {
            Err(Error::Trireme("Environment doesn't exist".to_string()))
        }
    }

    /// Delete specified environment
    pub fn remove_env(&mut self, env_name: &str) -> Result<()> {
        if self.envs.contains(&(env_name.to_string())) {
            self.envs.retain(|env| env != env_name);
            if let Some(env) = &self.current_env {
                if env == env_name {
                    self.current_env = None;
                }
            }
            Ok(())
        } else {
            Err(Error::Trireme("Environment doesn't exist".to_string()))
        }
    }

    /// Getter for the current environment name
    pub fn current_env(&self) -> Option<String> {
        self.current_env.clone()
    }

    /// Getter for list of environment names
    pub fn envs(&self) -> Vec<String> {
        self.envs.clone()
    }
}

/// Single environment client config
#[derive(Deserialize, Serialize)]
pub struct ClientConfig {
    name: String,
    variant: ClientVariant,
}

/// Variant of client config
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClientVariant {
    /// CML (Cardano Multiplatform Library) client
    CML(CMLClientConfig),
    /// Test client
    Test(TestClientConfig),
}

/// Config for CML client
#[derive(Deserialize, Serialize, Clone)]
pub struct CMLClientConfig {
    ledger_source: LedgerSource,
    key_source: KeySource,
    network: Network,
}

/// Config for Test client
#[derive(Deserialize, Serialize, Clone)]
pub struct TestClientConfig {
    data_path: PathBuf,
}

impl TestClientConfig {
    /// Getter for the data path
    pub fn data_path(&self) -> PathBuf {
        self.data_path.clone()
    }
}

impl ClientConfig {
    /// Constructor for CML client config
    pub fn new_cml(
        name: &str,
        ledger_source: LedgerSource,
        key_source: KeySource,
        network: Network,
    ) -> Self {
        let inner = CMLClientConfig {
            ledger_source,
            key_source,
            network,
        };
        let variant = ClientVariant::CML(inner);
        ClientConfig {
            name: name.to_string(),
            variant,
        }
    }

    /// Constructor for Test client config
    pub fn new_test(name: &str, data_path: &PathBuf) -> Self {
        let inner = TestClientConfig {
            data_path: data_path.to_owned(),
        };
        let variant = ClientVariant::Test(inner);
        ClientConfig {
            name: name.to_string(),
            variant,
        }
    }

    /// Getter for the name of config
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Getter for the variant of config
    pub fn variant(&self) -> ClientVariant {
        self.variant.clone()
    }

    /// Convert config into a [`TriremeLedgerClient`]
    pub async fn to_client<
        Datum: PlutusDataInterop
            + Clone
            + Eq
            + PartialEq
            + Debug
            + Hash
            + Send
            + Sync
            + Into<PlutusData>
            + TryFrom<PlutusData>,
        Redeemer: PlutusDataInterop + Clone + Eq + PartialEq + Debug + Hash + Send + Sync,
    >(
        self,
    ) -> Result<TriremeLedgerClient<Datum, Redeemer>> {
        match self.variant {
            ClientVariant::CML(inner) => {
                let network = inner.network;
                let keys = match inner.key_source {
                    KeySource::RawSecretPhrase { phrase_file } => {
                        let keys = RawSecretPhraseKeys::new(phrase_file, network.clone().into());
                        SecretPhraseKeys::RawSecretPhraseKeys(keys)
                    }
                    KeySource::TerminalPasswordUpfrontSecretPhrase {
                        phrase_file,
                        password_salt,
                        encrpytion_nonce,
                    } => {
                        let password = TerminalPasswordUpfront::init(&password_salt)?;
                        let keys = PasswordProtectedPhraseKeys::new(
                            password,
                            phrase_file,
                            network.clone().into(),
                            encrpytion_nonce,
                        );
                        SecretPhraseKeys::PasswordProtectedPhraseKeys(keys)
                    }
                };
                let inner_client = match inner.ledger_source {
                    LedgerSource::BlockFrost { api_key_file } => {
                        let blockfrost_key =
                            read_toml_struct_from_file::<BlockfrostApiKey>(&api_key_file)
                                .await?
                                .ok_or_else(|| {
                                    Error::Trireme(
                                "Couldn't find blockfrost config, please try reinitialize Trireme"
                                    .to_string(),
                            )
                                })?;
                        let key: String = blockfrost_key.into();
                        let url = match &network {
                            Network::Preprod => PREPROD_NETWORK_URL,
                            Network::Mainnet => MAINNET_URL,
                            Network::Preview => {
                                return Err(Error::Trireme(
                                    "Preview network not supported yet".to_string(),
                                ))
                            }
                        };
                        let ledger = BlockFrostLedger::new(url, &key);
                        let network_settings = network.clone().into();
                        InnerClient::BlockFrost(CMLLedgerCLient::new(
                            ledger,
                            keys,
                            network_settings,
                        ))
                    }
                    LedgerSource::OgmiosAndScrolls {
                        scrolls_ip,
                        scrolls_port,
                        ogmios_ip,
                        ogmios_port,
                    } => {
                        let scrolls_client = ScrollsClient::new_redis(scrolls_ip, scrolls_port);
                        let ogmios_client = OgmiosClient::new(ogmios_ip, ogmios_port);
                        let network_settings: NetworkSettings = network.into();
                        let ledger = OgmiosScrollsLedger::new(
                            scrolls_client,
                            ogmios_client,
                            network_settings,
                        );
                        InnerClient::OgmiosScrolls(CMLLedgerCLient::new(
                            ledger,
                            keys,
                            network_settings,
                        ))
                    }
                };

                let trireme_client = TriremeLedgerClient {
                    _datum: Default::default(),
                    _redeemer: Default::default(),
                    inner_client,
                };
                Ok(trireme_client)
            }
            ClientVariant::Test(inner) => {
                let data_dir = inner.data_path;
                let test_client = TestLedgerClient::load_local_persisted(data_dir);
                let inner_client = InnerClient::Mocked(test_client);
                let trireme_client = TriremeLedgerClient {
                    _datum: Default::default(),
                    _redeemer: Default::default(),
                    inner_client,
                };
                Ok(trireme_client)
            }
        }
    }
}

/// Variants of Secret Phrase [`Keys`] impl
pub enum SecretPhraseKeys {
    RawSecretPhraseKeys(RawSecretPhraseKeys),
    PasswordProtectedPhraseKeys(PasswordProtectedPhraseKeys<TerminalPasswordUpfront>),
}

#[async_trait]
impl Keys for SecretPhraseKeys {
    async fn base_addr(&self) -> cml_client::error::Result<BaseAddress> {
        match self {
            SecretPhraseKeys::RawSecretPhraseKeys(keys) => keys.base_addr().await,
            SecretPhraseKeys::PasswordProtectedPhraseKeys(keys) => keys.base_addr().await,
        }
    }

    async fn private_key(&self) -> cml_client::error::Result<PrivateKey> {
        match self {
            SecretPhraseKeys::RawSecretPhraseKeys(keys) => keys.private_key().await,
            SecretPhraseKeys::PasswordProtectedPhraseKeys(keys) => keys.private_key().await,
        }
    }
}

enum InnerClient<Datum, Redeemer>
where
    Datum: PlutusDataInterop
        + Clone
        + Send
        + Sync
        + PartialEq
        + Into<PlutusData>
        + TryFrom<PlutusData>,
    Redeemer: PlutusDataInterop,
{
    BlockFrost(CMLLedgerCLient<BlockFrostLedger, SecretPhraseKeys, Datum, Redeemer>),
    OgmiosScrolls(CMLLedgerCLient<OgmiosScrollsLedger, SecretPhraseKeys, Datum, Redeemer>),
    Mocked(TestLedgerClient<Datum, Redeemer, LocalPersistedStorage<PathBuf, Datum>>),
}

/// Implementation of [`LedgerClient`] for the Trireme CLI environment manager
pub struct TriremeLedgerClient<Datum, Redeemer>
where
    Datum: PlutusDataInterop
        + Clone
        + Send
        + Sync
        + PartialEq
        + Into<PlutusData>
        + TryFrom<PlutusData>,
    Redeemer: PlutusDataInterop,
{
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
    inner_client: InnerClient<Datum, Redeemer>,
}

impl<Datum, Redeemer> TriremeLedgerClient<Datum, Redeemer>
where
    Datum: PlutusDataInterop
        + Clone
        + Send
        + Sync
        + PartialEq
        + Into<PlutusData>
        + TryFrom<PlutusData>,
    Redeemer: PlutusDataInterop,
{
    pub async fn current_time(&self) -> LedgerClientResult<i64> {
        match &self.inner_client {
            InnerClient::BlockFrost(_cml_client) => Err(LedgerClientError::CurrentTime(Box::new(
                Error::Trireme("Not implemented for Blockfrost client".to_string()),
            ))),
            InnerClient::Mocked(test_client) => test_client.current_time_secs().await,
            InnerClient::OgmiosScrolls(_) => Err(LedgerClientError::CurrentTime(Box::new(
                Error::Trireme("Not implemented for Ogmios/Scrolls client".to_string()),
            ))),
        }
    }

    pub async fn advance_blocks(&self, count: i64) -> LedgerClientResult<()> {
        match &self.inner_client {
            InnerClient::BlockFrost(_cml_client) => Err(LedgerClientError::CurrentTime(Box::new(
                Error::Trireme("Not implemented for Blockfrost client".to_string()),
            ))),
            InnerClient::Mocked(test_client) => test_client.advance_time_n_blocks(count).await,
            InnerClient::OgmiosScrolls(_) => Err(LedgerClientError::CurrentTime(Box::new(
                Error::Trireme("Not implemented for Ogmios/Scrolls client".to_string()),
            ))),
        }
    }
}

#[async_trait]
impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for TriremeLedgerClient<Datum, Redeemer>
where
    Datum: PlutusDataInterop
        + Clone
        + Send
        + Sync
        + Debug
        + PartialEq
        + Into<PlutusData>
        + TryFrom<PlutusData>,
    Redeemer: PlutusDataInterop + Send + Sync + Clone + Eq + PartialEq + Debug + Hash,
{
    async fn signer_base_address(&self) -> LedgerClientResult<Address> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.signer_base_address(),
            InnerClient::Mocked(test_client) => test_client.signer_base_address(),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.signer_base_address(),
        }
        .await
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
        count: usize,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.outputs_at_address(address, count),
            InnerClient::Mocked(test_client) => test_client.outputs_at_address(address, count),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.outputs_at_address(address, count),
        }
        .await
    }

    async fn all_outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.all_outputs_at_address(address),
            InnerClient::Mocked(test_client) => test_client.all_outputs_at_address(address),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.all_outputs_at_address(address),
        }
        .await
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.issue(tx),
            InnerClient::Mocked(test_client) => test_client.issue(tx),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.issue(tx),
        }
        .await
    }

    async fn network(&self) -> LedgerClientResult<pallas_addresses::Network> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.network(),
            InnerClient::Mocked(test_client) => test_client.network(),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.network(),
        }
        .await
    }

    async fn last_block_time_secs(&self) -> LedgerClientResult<i64> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.last_block_time_secs(),
            InnerClient::Mocked(test_client) => test_client.last_block_time_secs(),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.last_block_time_secs(),
        }
        .await
    }

    async fn current_time_secs(&self) -> LedgerClientResult<i64> {
        match &self.inner_client {
            InnerClient::BlockFrost(cml_client) => cml_client.current_time_secs(),
            InnerClient::Mocked(test_client) => <TestLedgerClient<
                Datum,
                Redeemer,
                LocalPersistedStorage<PathBuf, Datum>,
            > as LedgerClient<Datum, Redeemer>>::current_time_secs(
                test_client
            ),
            InnerClient::OgmiosScrolls(cml_client) => cml_client.current_time_secs(),
        }
        .await
    }
}

#[derive(Debug, Error)]
pub enum TomlError {
    #[error("No config directory for raw phrase file: {0:?}")]
    NoParentDir(String),
}

pub async fn write_toml_struct_to_file<Toml: ser::Serialize>(
    file_path: &PathBuf,
    toml_struct: &Toml,
) -> Result<()> {
    let serialized = toml::to_string(&toml_struct).map_err(|e| Error::TOML(Box::new(e)))?;
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| TomlError::NoParentDir(format!("{file_path:?}")))
        .map_err(|e| Error::TOML(Box::new(e)))?;
    fs::create_dir_all(&parent_dir)
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&file_path)
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    file.write_all(&serialized.into_bytes())
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    Ok(())
}

pub async fn read_toml_struct_from_file<Toml: DeserializeOwned>(
    file_path: &PathBuf,
) -> Result<Option<Toml>> {
    if file_path.exists() {
        let contents = fs::read_to_string(file_path)
            .await
            .map_err(|e| Error::TOML(Box::new(e)))?;
        let toml_struct = toml::from_str(&contents).map_err(|e| Error::TOML(Box::new(e)))?;
        Ok(Some(toml_struct))
    } else {
        Ok(None)
    }
}
