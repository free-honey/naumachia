use super::{error::*, Keys};
use async_trait::async_trait;
use bip39::{Language, Mnemonic};
use cardano_multiplatform_lib::{
    address::{Address as CMLAddress, BaseAddress, StakeCredential},
    crypto::{Bip32PrivateKey, PrivateKey},
};
use std::{fs, path::Path};
use thiserror::Error;

pub const TESTNET: u8 = 0;
pub const MAINNET: u8 = 1;

pub struct KeyManager {
    config_path: String,
    network: u8,
}

impl KeyManager {
    pub fn new(config_path: String, network: u8) -> Self {
        KeyManager {
            config_path,
            network,
        }
    }
}

#[derive(Debug, Error)]
pub enum KeyManagerError {
    #[error("Some non-StdError 🤮 error from Bip32 lib: {0:?}")]
    Bip39(String),
}

#[async_trait]
impl Keys for KeyManager {
    async fn base_addr(&self) -> Result<BaseAddress> {
        let base_addr = self.get_base_address()?;
        Ok(base_addr)
    }

    async fn private_key(&self) -> Result<PrivateKey> {
        let account_key = self.get_account_key()?;
        let priv_key = account_key.derive(0).derive(0).to_raw_key();
        Ok(priv_key)
    }

    async fn addr_from_bech_32(&self, addr: &str) -> Result<CMLAddress> {
        let cml_address =
            CMLAddress::from_bech32(addr).map_err(|e| CMLLCError::JsError(e.to_string()))?;
        Ok(cml_address)
    }
}

impl KeyManager {
    fn get_account_key(&self) -> Result<Bip32PrivateKey> {
        let phrase = load_phrase_from_file(&self.config_path);
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)
            .map_err(|e| KeyManagerError::Bip39(e.to_string()))
            .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
        let entropy = mnemonic.entropy();
        let root_key = Bip32PrivateKey::from_bip39_entropy(entropy, &[]);

        let account_key = root_key
            .derive(harden(1852))
            .derive(harden(1815))
            .derive(harden(0));
        Ok(account_key)
    }

    fn get_base_address(&self) -> Result<BaseAddress> {
        let account_key = self.get_account_key()?;
        let pub_key = account_key.derive(0).derive(0).to_public();
        let stake_key = account_key.derive(2).derive(0).to_public();
        let pub_key_creds = StakeCredential::from_keyhash(&pub_key.to_raw_key().hash());
        let stake_key_creds = StakeCredential::from_keyhash(&stake_key.to_raw_key().hash());
        let base_addr = BaseAddress::new(self.network, &pub_key_creds, &stake_key_creds);
        Ok(base_addr)
    }
}

pub fn load_phrase_from_file(config_path: &str) -> String {
    let path = Path::new(config_path);
    let text = fs::read_to_string(&path).unwrap();
    let config: toml::Value = toml::from_str(&text).unwrap();
    config["phrase"].as_str().unwrap().to_string()
}

fn harden(index: u32) -> u32 {
    index | 0x80_00_00_00
}
