use crate::ledger_client::blockfrost_client::blockfrost_http_client::schemas::{
    Address, AddressInfo, Genesis, UTxO,
};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;
use thiserror::Error;
use url::Url;

pub mod schemas;

const TEST_URL: &str = "https://cardano-testnet.blockfrost.io/api/v0/";
// Must include a TOML file at your project root with the field:
//   project_id = <INSERT API KEY HERE>
const CONFIG_PATH: &str = ".blockfrost.toml";

fn load_key_from_file(key_path: &str) -> String {
    let path = Path::new(key_path);
    let text = fs::read_to_string(&path).unwrap();
    let config: toml::Value = toml::from_str(&text).unwrap();
    config["project_id"].as_str().unwrap().to_string()
}

pub fn get_test_bf_http_clent() -> BlockfrostHttp {
    let key = load_key_from_file(CONFIG_PATH);
    BlockfrostHttp::new(TEST_URL, &key)
}

pub struct BlockfrostHttp {
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
}

impl BlockfrostHttp {
    pub fn new(url: &str, key: &str) -> Self {
        let parent_url = url.to_string();
        let api_key = key.to_string();
        BlockfrostHttp {
            parent_url,
            api_key,
        }
    }

    pub async fn genesis(&self) -> Result<Genesis> {
        let ext = "./genesis";
        self.get_endpoint(ext).await
    }

    pub async fn address_info(&self, address: &str) -> Result<AddressInfo> {
        let ext = format!("./addresses/{}", address);
        self.get_endpoint(&ext).await
    }

    pub async fn utxos(&self, address: &str) -> Result<Vec<UTxO>> {
        let ext = format!("./addresses/{}/utxos", address);
        self.get_endpoint(&ext).await
    }

    pub async fn datum(&self, datum_hash: &str) -> Result<serde_json::Value> {
        let ext = format!("./scripts/datum/{}", datum_hash);
        self.get_endpoint(&ext).await
    }

    pub async fn assoc_addresses(&self, stake_address: &str) -> Result<Vec<Address>> {
        let ext = format!("./accounts/{}/addresses", stake_address);
        self.get_endpoint(&ext).await
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::ledger_client::blockfrost_client::keys::TESTNET;
    use crate::ledger_client::blockfrost_client::tests::my_base_addr;
    use cardano_multiplatform_lib::address::RewardAddress;
    use std::fs;
    use std::path::Path;

    #[ignore]
    #[tokio::test]
    async fn genesis() -> Result<()> {
        let bf = get_test_bf_http_clent();
        let _res = bf.genesis().await.unwrap();
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn utxos() -> Result<()> {
        let bf = get_test_bf_http_clent();
        // TODO: Find a good stable address to use
        // let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";
        // let address = "addr_test1wrsexavz37208qda7mwwu4k7hcpg26cz0ce86f5e9kul3hqzlh22t";
        let address = "addr_test1wp9m8xkpt2tmy7madqldspgzgug8f2p3pwhz589cq75685slenwf4";
        let res = bf.utxos(&address).await.unwrap();
        dbg!(&res);
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn datum() -> Result<()> {
        let bf = get_test_bf_http_clent();
        // TODO: Find a good stable address to use
        // let datum_hash = "d1cede40100329bfd7edbb1245a4d24de23924f00341886dc5f5bf6d06c65629";
        let datum_hash = "a9fbe52ace8f89e0ae64d88f879e159b97d51f27d8f932c9aa165e5ce5f0f28e";
        let res = bf.datum(&datum_hash).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&res).unwrap());
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn address_info() -> Result<()> {
        let bf = get_test_bf_http_clent();
        // let address = "addr1q97dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fsmqn6vh";
        // let address = "addr1qp7dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fs2k6a72";
        let address = "addr_test1wrtlw9csk7vc9peauh9nzpg45zemvj3w9m532e93nwer24gjwycdl";

        let res = bf.address_info(&address).await.unwrap();
        dbg!(&res);
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn account_associated_addresses() {
        let bf = get_test_bf_http_clent();
        let base_addr = my_base_addr();
        let staking_cred = base_addr.stake_cred();

        let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
            .to_address()
            .to_bech32(None)
            .unwrap();
        let res = bf.assoc_addresses(&reward_addr).await.unwrap();
        dbg!(&res);
    }

    #[ignore]
    #[tokio::test]
    async fn account_associated_addresses_total() {
        let bf = get_test_bf_http_clent();
        let base_addr = my_base_addr();
        let staking_cred = base_addr.stake_cred();

        let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
            .to_address()
            .to_bech32(None)
            .unwrap();
        let res = bf
            .account_associated_addresses_total(&reward_addr)
            .await
            .unwrap();
        dbg!(&res);
    }
}
