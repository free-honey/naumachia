use crate::trireme_ledger_client::{
    cml_client::{
        error::{
            CMLLCError,
            Result as CMLLCResult,
        },
        Keys,
    },
    secret_phrase::{
        private_key_to_base_address,
        secret_phrase_to_account_key,
    },
};
use async_trait::async_trait;
use cardano_multiplatform_lib::{
    address::BaseAddress,
    crypto::{
        Bip32PrivateKey,
        PrivateKey,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    path::PathBuf,
    str::FromStr,
};
use thiserror::Error;
use tokio::fs;

/// Type for holding the raw secret phrase
pub struct RawSecretPhraseKeys {
    phrase_file_path: PathBuf,
    network: u8,
}

impl RawSecretPhraseKeys {
    /// Constructor for the [`RawSecretPhraseKeys`] struct
    pub fn new(phrase_file_path: PathBuf, network: u8) -> Self {
        RawSecretPhraseKeys {
            phrase_file_path,
            network,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum RawSecretPhraseKeysError {
    #[error("Some non-StdError 🤮 error from Bip39 lib: {0:?}")]
    Bip39(String),
    #[error("No config directory for raw phrase file: {0:?}")]
    NoConfigDirectory(String),
}

impl RawSecretPhraseKeys {
    /// Get the account key from the secret phrase
    async fn get_account_key(&self) -> CMLLCResult<Bip32PrivateKey> {
        let phrase: String = read_secret_phrase(&self.phrase_file_path).await?.into();
        let account_key = secret_phrase_to_account_key(&phrase)?;
        Ok(account_key)
    }

    /// Get the base address from the secret phrase
    async fn get_base_address(&self) -> CMLLCResult<BaseAddress> {
        let account_key = self.get_account_key().await?;
        let base_addr = private_key_to_base_address(&account_key, self.network);
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
}

/// Type for holding the secret phrase
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.to_string();
        Ok(SecretPhrase { inner })
    }
}

/// Reads the secret phrase from the config file
pub async fn read_secret_phrase(config_path: &PathBuf) -> CMLLCResult<SecretPhrase> {
    let text = fs::read_to_string(config_path)
        .await
        .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
    toml::from_str(&text).map_err(|e| CMLLCError::KeyError(Box::new(e)))
}
