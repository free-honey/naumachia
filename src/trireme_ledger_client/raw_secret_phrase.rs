use crate::ledger_client::cml_client::error::{CMLLCError, Result as CMLLCResult};
use crate::ledger_client::cml_client::Keys;
use async_trait::async_trait;
use bip39::{Language, Mnemonic};
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress, StakeCredential};
use cardano_multiplatform_lib::crypto::{Bip32PrivateKey, PrivateKey};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use tokio::fs;

pub struct RawSecretPhraseKeys {
    phrase_file_path: PathBuf,
    network: u8,
}

impl RawSecretPhraseKeys {
    pub fn new(phrase_file_path: PathBuf, network: u8) -> Self {
        RawSecretPhraseKeys {
            phrase_file_path,
            network,
        }
    }
}

#[derive(Debug, Error)]
pub enum RawSecretPhraseKeysError {
    #[error("Some non-StdError 🤮 error from Bip39 lib: {0:?}")]
    Bip39(String),
    #[error("No config directory for raw phrase file: {0:?}")]
    NoConfigDirectory(String),
}

impl RawSecretPhraseKeys {
    async fn get_account_key(&self) -> CMLLCResult<Bip32PrivateKey> {
        let phrase: String = read_secret_phrase(&self.phrase_file_path).await?.into();
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)
            .map_err(|e| RawSecretPhraseKeysError::Bip39(e.to_string()))
            .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
        let entropy = mnemonic.entropy();
        let root_key = Bip32PrivateKey::from_bip39_entropy(entropy, &[]);

        let account_key = root_key
            .derive(harden(1852))
            .derive(harden(1815))
            .derive(harden(0));
        Ok(account_key)
    }

    async fn get_base_address(&self) -> CMLLCResult<BaseAddress> {
        let account_key = self.get_account_key().await?;
        let pub_key = account_key.derive(0).derive(0).to_public();
        let stake_key = account_key.derive(2).derive(0).to_public();
        let pub_key_creds = StakeCredential::from_keyhash(&pub_key.to_raw_key().hash());
        let stake_key_creds = StakeCredential::from_keyhash(&stake_key.to_raw_key().hash());
        let base_addr = BaseAddress::new(self.network, &pub_key_creds, &stake_key_creds);
        Ok(base_addr)
    }
}

#[async_trait]
impl Keys for RawSecretPhraseKeys {
    async fn base_addr(&self) -> CMLLCResult<BaseAddress> {
        let base_addr = self.get_base_address().await?;
        Ok(base_addr)
    }

    async fn private_key(&self) -> CMLLCResult<PrivateKey> {
        let account_key = self.get_account_key().await?;
        let priv_key = account_key.derive(0).derive(0).to_raw_key();
        Ok(priv_key)
    }

    async fn addr_from_bech_32(&self, addr: &str) -> CMLLCResult<CMLAddress> {
        let cml_address =
            CMLAddress::from_bech32(addr).map_err(|e| CMLLCError::JsError(e.to_string()))?;
        Ok(cml_address)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SecretPhrase {
    inner: String,
}

impl From<SecretPhrase> for String {
    fn from(secret_phrase: SecretPhrase) -> Self {
        secret_phrase.inner
    }
}

impl From<&SecretPhrase> for String {
    fn from(secret_phrase: &SecretPhrase) -> Self {
        secret_phrase.inner.clone()
    }
}

impl FromStr for SecretPhrase {
    type Err = RawSecretPhraseKeysError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let inner = s.to_string();
        Ok(SecretPhrase { inner })
    }
}

pub async fn read_secret_phrase(config_path: &PathBuf) -> CMLLCResult<SecretPhrase> {
    let text = fs::read_to_string(config_path)
        .await
        .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
    toml::from_str(&text).map_err(|e| CMLLCError::KeyError(Box::new(e)))
}

fn harden(index: u32) -> u32 {
    index | 0x80_00_00_00
}
