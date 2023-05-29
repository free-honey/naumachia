use crate::error::Error;
use crate::error::Result;
use crate::trireme_ledger_client::cml_client::Keys;
use async_trait::async_trait;
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress};
use cardano_multiplatform_lib::crypto::PrivateKey;
use dialoguer::Password as InputPassword;
use secrecy::{ExposeSecret, Secret};
use std::path::PathBuf;

pub const SALT: &[u8] = b"brackish water";

pub struct PasswordProtectedPhraseKeys<P: Password> {
    password: P,
    phrase_file_path: PathBuf,
    network: u8,
}

impl<P: Password> PasswordProtectedPhraseKeys<P> {
    pub fn new(password: P, phrase_file_path: PathBuf, network_index: u8) -> Self {
        Self {
            password,
            phrase_file_path,
            network: network_index,
        }
    }
}

#[async_trait]
impl<P: Password + Send + Sync> Keys for PasswordProtectedPhraseKeys<P> {
    async fn base_addr(
        &self,
    ) -> crate::trireme_ledger_client::cml_client::error::Result<BaseAddress> {
        todo!()
    }

    async fn private_key(
        &self,
    ) -> crate::trireme_ledger_client::cml_client::error::Result<PrivateKey> {
        todo!()
    }

    async fn addr_from_bech_32(
        &self,
        addr: &str,
    ) -> crate::trireme_ledger_client::cml_client::error::Result<CMLAddress> {
        todo!()
    }
}

pub struct TerminalPasswordUpfront {
    password: Secret<String>,
}

impl TerminalPasswordUpfront {
    /// Reads the password from the terminal and hashes it with Argon2.
    pub fn init() -> Result<Self> {
        let password = InputPassword::new()
            .with_prompt("Enter password")
            .interact()
            .map_err(|e| Error::Trireme(e.to_string()))?;
        let config = argon2::Config::default();
        let hashed = argon2::hash_encoded(password.as_bytes(), SALT, &config)
            .map_err(|e| Error::Trireme(e.to_string()))?;
        let secret = Secret::new(hashed);
        let password_container = Self { password: secret };
        Ok(password_container)
    }
}

pub trait Password {
    fn get_password(&self) -> String;
}

impl Password for TerminalPasswordUpfront {
    fn get_password(&self) -> String {
        self.password.expose_secret().to_owned()
    }
}
