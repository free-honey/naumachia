use crate::{
    error::*,
    ledger_client::{
        cml_client::{
            blockfrost_ledger::BlockFrostLedger, key_manager::KeyManager,
            plutus_data_interop::PlutusDataInterop, CMLLedgerCLient,
        },
        LedgerClient, LedgerClientResult,
    },
    output::Output,
    transaction::TxId,
    trireme_ledger_client::raw_secret_phrase::RawSecretPhraseKeys,
    Address, UnbuiltTransaction,
};
use async_trait::async_trait;
use dirs::home_dir;
use serde::ser;
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, path::Path, path::PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub mod blockfrost_ledger;
pub mod raw_secret_phrase;

pub const TRIREME_CONFIG_FOLDER: &str = ".trireme";

pub fn path_to_trireme_config_dir() -> Result<PathBuf> {
    let mut dir = home_dir().ok_or(Error::Trireme(
        "Could not find home directory :(".to_string(),
    ))?;
    dir.push(TRIREME_CONFIG_FOLDER);
    Ok(dir)
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LedgerSource {
    BlockFrost { api_key_file: PathBuf },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum KeySource {
    RawSecretPhrase { phrase_file: PathBuf },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Network {
    Testnet,
    Mainnet,
}

#[derive(Deserialize, Serialize)]
pub struct TriremeConfig {
    ledger_source: LedgerSource,
    key_source: KeySource,
    network: Network,
}

impl TriremeConfig {
    pub fn new(ledger_source: LedgerSource, key_source: KeySource, network: Network) -> Self {
        TriremeConfig {
            ledger_source,
            key_source,
            network,
        }
    }

    pub fn to_client<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        self,
        network: u8,
    ) -> Result<TriremeLedgerClient<Datum, Redeemer>> {
        let ledger = match self.ledger_source {
            LedgerSource::BlockFrost { api_key_file } => {
                todo!()
            }
        };
        let keys = match self.key_source {
            KeySource::RawSecretPhrase { phrase_file } => {
                RawSecretPhraseKeys::new(phrase_file, network)
            }
        };
        let inner_client = InnerClient::CML(CMLLedgerCLient::new(ledger, keys, network));
        let trireme_client = TriremeLedgerClient {
            _datum: Default::default(),
            _redeemer: Default::default(),
            inner_client,
        };
        Ok(trireme_client)
    }
}

enum InnerClient<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop> {
    CML(CMLLedgerCLient<BlockFrostLedger, RawSecretPhraseKeys, Datum, Redeemer>),
}

pub struct TriremeLedgerClient<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop> {
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
    inner_client: InnerClient<Datum, Redeemer>,
}

#[async_trait]
impl<Datum: PlutusDataInterop + Send + Sync, Redeemer: PlutusDataInterop + Send + Sync>
    LedgerClient<Datum, Redeemer> for TriremeLedgerClient<Datum, Redeemer>
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.signer(),
        }
        .await
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.outputs_at_address(address),
        }
        .await
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.issue(tx),
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
        .ok_or(TomlError::NoParentDir(format!("{:?}", file_path)))
        .map_err(|e| Error::TOML(Box::new(e)))?;
    fs::create_dir_all(&parent_dir)
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_path)
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    file.write_all(&serialized.into_bytes())
        .await
        .map_err(|e| Error::TOML(Box::new(e)))?;
    Ok(())
}
