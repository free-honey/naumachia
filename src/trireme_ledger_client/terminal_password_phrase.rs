use crate::error::Error;
use crate::error::Result;
use crate::trireme_ledger_client::cml_client::Keys;
use async_trait::async_trait;
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress};
use cardano_multiplatform_lib::crypto::PrivateKey;
use chacha20::{
    cipher::{KeyIvInit, StreamCipher, StreamCipherSeek},
    ChaCha20,
};
use dialoguer::Password as InputPassword;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

pub const SALT: &[u8] = b"brackish water";

pub struct PasswordProtectedPhraseKeys<P: Password> {
    password: P,
    phrase_file_path: PathBuf,
    nonce: [u8; 12],
    network: u8,
}

impl<P: Password> PasswordProtectedPhraseKeys<P> {
    // TODO: Add nonce
    pub fn new(password: P, phrase_file_path: PathBuf, network_index: u8) -> Self {
        let nonce = [0; 12];

        Self {
            password,
            phrase_file_path,
            nonce,
            network: network_index,
        }
    }

    pub async fn read_phrase(&self) -> Result<String> {
        let text = fs::read_to_string(&self.phrase_file_path).await.unwrap();
        let encrypted: EncryptedSecretPhrase = toml::from_str(&text).unwrap();
        let password = self.password.get_password().unwrap();
        let phrase = decrypt_phrase(&encrypted, &password);
        Ok(phrase)
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
    password: Secret<[u8; 32]>,
}

impl TerminalPasswordUpfront {
    /// Reads the password from the terminal and hashes it with Argon2.
    pub fn init() -> Result<Self> {
        let password = InputPassword::new()
            .with_prompt("Enter password")
            .interact()
            .map_err(|e| Error::Trireme(e.to_string()))?;

        let new = normalize_password(&password)?;
        let secret = Secret::new(new);
        let password_container = Self { password: secret };
        Ok(password_container)
    }
}

fn normalize_password(original: &str) -> Result<[u8; 32]> {
    // TODO: Upgrade config to be more secure?
    let config = argon2::Config::default();
    let hashed = argon2::hash_raw(original.as_bytes(), SALT, &config)
        .map_err(|e| Error::Trireme(e.to_string()))?;
    // TODO: Verify that the argon2 output is always long enough
    let new = hashed[..32].try_into().expect("always correct length");
    Ok(new)
}

pub trait Password {
    fn get_password(&self) -> Result<[u8; 32]>;
}

impl Password for TerminalPasswordUpfront {
    fn get_password(&self) -> Result<[u8; 32]> {
        let password = self.password.expose_secret().to_owned();
        Ok(password)
    }
}

fn encrypt_phrase(phrase: &str, password: &[u8; 32]) -> EncryptedSecretPhrase {
    let key = password;
    let nonce = [0u8; 12];
    let mut cipher = ChaCha20::new(key.into(), &nonce.into());
    let mut buffer: Vec<_> = phrase.bytes().collect();

    cipher.apply_keystream(&mut buffer);

    EncryptedSecretPhrase { inner: buffer }
}

fn decrypt_phrase(encrypted_phrase: &EncryptedSecretPhrase, password: &[u8; 32]) -> String {
    let key = password;
    let nonce = [0u8; 12];
    let mut cipher = ChaCha20::new(key.into(), &nonce.into());
    let mut buffer: Vec<_> = encrypted_phrase.inner.clone();

    cipher.apply_keystream(&mut buffer);

    String::from_utf8(buffer).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedSecretPhrase {
    inner: Vec<u8>,
}

impl From<EncryptedSecretPhrase> for String {
    fn from(secret_phrase: EncryptedSecretPhrase) -> Self {
        format!("{:?}", secret_phrase.inner)
    }
}

impl From<&EncryptedSecretPhrase> for String {
    fn from(secret_phrase: &EncryptedSecretPhrase) -> Self {
        format!("{:?}", secret_phrase.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trireme_ledger_client::write_toml_struct_to_file;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    struct InMemoryPassword {
        password: [u8; 32],
    }

    impl InMemoryPassword {
        pub fn new(password: [u8; 32]) -> Self {
            Self { password }
        }

        pub fn encrypt_phrase(&self, phrase: &str) -> EncryptedSecretPhrase {
            let password = self.get_password().unwrap();
            encrypt_phrase(phrase, &password)
        }
    }

    impl Password for InMemoryPassword {
        fn get_password(&self) -> Result<[u8; 32]> {
            Ok(self.password.clone())
        }
    }

    #[tokio::test]
    async fn roundtrip_phrase() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("my-temporary-note.txt");
        let mut file = File::create(&file_path).unwrap();

        let original_phrase =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon";

        let nonce = [0x24; 12];

        let normalized = normalize_password("password").unwrap();

        let password = InMemoryPassword::new(normalized);

        let encrypted_phrase = password.encrypt_phrase(original_phrase);

        write_toml_struct_to_file(&file_path, &encrypted_phrase)
            .await
            .unwrap();

        let keys = PasswordProtectedPhraseKeys::new(password, file_path, 0);

        let new_phrase = keys.read_phrase().await.unwrap();

        assert_eq!(original_phrase, &new_phrase);
    }
}
