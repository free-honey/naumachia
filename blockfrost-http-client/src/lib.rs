use crate::schemas::{Address, AddressInfo, Genesis, UTxO};
use async_trait::async_trait;
use cardano_multiplatform_lib::address::{BaseAddress, StakeCredential};
use serde::de::DeserializeOwned;
use std::path::Path;
use std::{fs, io};
use thiserror::Error;
use url::Url;

use bip39::{Language, Mnemonic};
use cardano_multiplatform_lib::crypto::Bip32PrivateKey;

pub mod schemas;
#[cfg(test)]
pub mod tests;

pub const TESTNET: u8 = 0;
pub const MAINNET: u8 = 1;

const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

pub fn my_base_addr() -> BaseAddress {
    let phrase = load_phrase_from_file(CONFIG_PATH);
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

    let entropy = mnemonic.entropy();

    base_address_from_entropy(entropy, TESTNET)
}

pub fn load_phrase_from_file(config_path: &str) -> String {
    let path = Path::new(config_path);
    let text = fs::read_to_string(&path).unwrap();
    let config: toml::Value = toml::from_str(&text).unwrap();
    config["phrase"].as_str().unwrap().to_string()
}

pub fn base_address_from_entropy(entropy: &[u8], network: u8) -> BaseAddress {
    fn harden(index: u32) -> u32 {
        index | 0x80_00_00_00
    }

    let root_key = Bip32PrivateKey::from_bip39_entropy(entropy, &[]);

    let account_key = root_key
        .derive(harden(1852))
        .derive(harden(1815))
        .derive(harden(0));

    let pub_key = account_key.derive(0).derive(0).to_public();
    let stake_key = account_key.derive(2).derive(0).to_public();

    let pub_key_creds = StakeCredential::from_keyhash(&pub_key.to_raw_key().hash());
    let stake_key_creds = StakeCredential::from_keyhash(&stake_key.to_raw_key().hash());

    BaseAddress::new(network, &pub_key_creds, &stake_key_creds)
}

fn load_key_from_file(key_path: &str) -> Result<String> {
    let path = Path::new(key_path);
    let text = fs::read_to_string(&path).map_err(Error::FileRead)?;
    let config: toml::Value = toml::from_str(&text).map_err(Error::Toml)?;
    let field = "project_id";
    let project_id = config[field]
        .as_str()
        .ok_or_else(|| Error::Config(field.to_string()))?
        .to_string();
    Ok(project_id)
}

pub fn get_test_bf_http_clent() -> Result<BlockFrostHttp> {
    let key = load_key_from_file(CONFIG_PATH)?;
    let bf = BlockFrostHttp::new(TEST_URL, &key);
    Ok(bf)
}

pub struct BlockFrostHttp {
    parent_url: String,
    api_key: String, // A.K.A. `project_id`
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url Error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Config field not found: {0:?}")]
    Config(String),
    #[error("Error while reading file: {0:?}")]
    FileRead(io::Error),
    #[error("Error while parsing Toml: {0:?}")]
    Toml(toml::de::Error),
}

#[async_trait]
pub trait BlockFrostHttpTrait {
    async fn genesis(&self) -> Result<Genesis>;

    async fn address_info(&self, address: &str) -> Result<AddressInfo>;

    async fn utxos(&self, address: &str) -> Result<Vec<UTxO>>;

    async fn datum(&self, datum_hash: &str) -> Result<serde_json::Value>;

    async fn assoc_addresses(&self, stake_address: &str) -> Result<Vec<Address>>;
}

#[async_trait]
impl BlockFrostHttpTrait for BlockFrostHttp {
    async fn genesis(&self) -> Result<Genesis> {
        let ext = "./genesis";
        self.get_endpoint(ext).await
    }

    async fn address_info(&self, address: &str) -> Result<AddressInfo> {
        let ext = format!("./addresses/{}", address);
        self.get_endpoint(&ext).await
    }

    async fn utxos(&self, address: &str) -> Result<Vec<UTxO>> {
        let ext = format!("./addresses/{}/utxos", address);
        self.get_endpoint(&ext).await
    }

    async fn datum(&self, datum_hash: &str) -> Result<serde_json::Value> {
        let ext = format!("./scripts/datum/{}", datum_hash);
        self.get_endpoint(&ext).await
    }

    async fn assoc_addresses(&self, stake_address: &str) -> Result<Vec<Address>> {
        let ext = format!("./accounts/{}/addresses", stake_address);
        self.get_endpoint(&ext).await
    }
}

impl BlockFrostHttp {
    pub fn new(url: &str, key: &str) -> Self {
        let parent_url = url.to_string();
        let api_key = key.to_string();
        BlockFrostHttp {
            parent_url,
            api_key,
        }
    }

    pub async fn execution_units(&self) -> Result<()> {
        todo!()
    }

    pub async fn account_associated_addresses_total(
        &self,
        base_addr: &str,
    ) -> Result<Vec<Address>> {
        // pub async fn account_associated_addresses(&self, base_addr: &str) -> Result<AccountAssocAddr> {
        let ext = format!("./accounts/{}/addresses/total", base_addr);
        dbg!(&ext);
        self.get_endpoint(&ext).await
    }

    async fn get_endpoint<T: DeserializeOwned>(&self, ext: &str) -> Result<T> {
        let url = Url::parse(&self.parent_url)?.join(ext)?;
        let client = reqwest::Client::new();
        let project_id = &self.api_key;
        let res = client
            .get(url)
            .header("project_id", project_id)
            .send()
            .await?;
        // dbg!(&res);
        let res = res.json().await?;
        Ok(res)
    }
}
