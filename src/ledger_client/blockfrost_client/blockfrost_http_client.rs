use crate::ledger_client::blockfrost_client::blockfrost_http_client::schemas::Genesis;
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

pub mod schemas;

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

    async fn get_endpoint<T: DeserializeOwned>(&self, ext: &str) -> Result<T> {
        let url = Url::parse(&self.parent_url)?.join(ext)?;
        let client = reqwest::Client::new();
        let project_id = &self.api_key;
        let res = client
            .get(url)
            .header("project_id", project_id)
            .send()
            .await?;
        dbg!(&res);
        let res = res.json().await?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

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

    fn get_test_bf_http_clent() -> BlockfrostHttp {
        let key = load_key_from_file(CONFIG_PATH);
        BlockfrostHttp::new(TEST_URL, &key)
    }

    #[ignore]
    #[tokio::test]
    async fn genesis() -> Result<()> {
        let bf = get_test_bf_http_clent();
        let res = bf.genesis().await.unwrap();
        Ok(())
    }
}
