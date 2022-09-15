use crate::models::{Address, AddressInfo, EvaluateTxResult, Genesis, ProtocolParams, UTxO};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::{fs, path::Path};
use url::Url;

use crate::error::{Error, Result};

pub mod error;
pub mod keys;
pub mod models;
#[cfg(test)]
pub mod tests;

// const MAINNET_URL: &str = "https://cardano-mainnet.blockfrost.io/api/v0";
const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

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

pub fn get_test_bf_http_client() -> Result<BlockFrostHttp> {
    let key = load_key_from_file(CONFIG_PATH)?;
    let bf = BlockFrostHttp::new(TEST_URL, &key);
    Ok(bf)
}

pub struct BlockFrostHttp {
    parent_url: String,
    api_key: String, // A.K.A. `project_id`
}

#[async_trait]
pub trait BlockFrostHttpTrait {
    async fn genesis(&self) -> Result<Genesis>;

    async fn protocol_params(&self, epoch: u32) -> Result<ProtocolParams>;

    async fn address_info(&self, address: &str) -> Result<AddressInfo>;

    async fn utxos(&self, address: &str) -> Result<Vec<UTxO>>;

    async fn datum(&self, datum_hash: &str) -> Result<serde_json::Value>;

    async fn assoc_addresses(&self, stake_address: &str) -> Result<Vec<Address>>;

    async fn account_associated_addresses_total(&self, base_addr: &str) -> Result<Vec<Address>>;

    async fn execution_units(&self, bytes: &[u8]) -> Result<EvaluateTxResult>;

    async fn submit_tx(&self, bytes: &[u8]) -> Result<serde_json::Value>;
}

#[async_trait]
impl BlockFrostHttpTrait for BlockFrostHttp {
    async fn genesis(&self) -> Result<Genesis> {
        let ext = "./genesis";
        self.get_endpoint(ext).await
    }

    async fn protocol_params(&self, epoch: u32) -> Result<ProtocolParams> {
        let ext = format!("./epochs/{}/parameters", epoch);
        self.get_endpoint(&ext).await
    }

    async fn address_info(&self, address: &str) -> Result<AddressInfo> {
        let ext = format!("./addresses/{}", address);
        self.get_endpoint(&ext).await
    }

    async fn utxos(&self, address: &str) -> Result<Vec<UTxO>> {
        let ext = format!("./addresses/{}/utxos", address);
        let params = [("order", "desc")];
        self.get_endpoint_with_params(&ext, &params).await
    }

    async fn datum(&self, datum_hash: &str) -> Result<serde_json::Value> {
        let ext = format!("./scripts/datum/{}", datum_hash);
        self.get_endpoint(&ext).await
    }

    async fn assoc_addresses(&self, stake_address: &str) -> Result<Vec<Address>> {
        let ext = format!("./accounts/{}/addresses", stake_address);
        self.get_endpoint(&ext).await
    }

    async fn account_associated_addresses_total(&self, base_addr: &str) -> Result<Vec<Address>> {
        let ext = format!("./accounts/{}/addresses/total", base_addr);
        dbg!(&ext);
        self.get_endpoint(&ext).await
    }

    async fn execution_units(&self, bytes: &[u8]) -> Result<EvaluateTxResult> {
        let ext = "./utils/txs/evaluate".to_string();
        let url = Url::parse(&self.parent_url)?.join(&ext)?;
        let client = reqwest::Client::new();
        let project_id = &self.api_key;
        let encoded = hex::encode(bytes);
        let res = client
            .post(url)
            .header("Content-Type", "application/cbor")
            .header("project_id", project_id)
            .body(encoded)
            .send()
            .await
            .unwrap();
        // let json_for_fun: serde_json::Value = res.json().await?;
        // dbg!(&json_for_fun);
        Ok(res.json().await?)
    }

    async fn submit_tx(&self, bytes: &[u8]) -> Result<serde_json::Value> {
        let ext = "./tx/submit".to_string();
        let url = Url::parse(&self.parent_url)?.join(&ext)?;
        let client = reqwest::Client::new();
        let project_id = &self.api_key;
        let res = client
            .post(url)
            .header("Content-Type", "application/cbor")
            .header("project_id", project_id)
            .body(bytes.to_owned()) // For some dumb-ass reason this is binary
            .send()
            .await
            .unwrap();
        Ok(res.json().await?)
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

    async fn get_endpoint<T: DeserializeOwned>(&self, ext: &str) -> Result<T> {
        self.get_endpoint_with_params(ext, &[]).await
    }

    async fn get_endpoint_with_params<T: DeserializeOwned>(
        &self,
        ext: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let mut url = Url::parse(&self.parent_url)?.join(ext)?;
        url.query_pairs_mut().extend_pairs(params);
        let client = reqwest::Client::new();
        let project_id = &self.api_key;
        let res = client
            .get(url)
            .header("project_id", project_id)
            .send()
            .await?;
        let res = res.json().await?;
        Ok(res)
    }
}
