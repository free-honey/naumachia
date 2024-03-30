use crate::{
    error::{
        Error,
        Result,
    },
    trireme_ledger_client::{
        cml_client::Keys,
        secret_phrase::{
            private_key_to_base_address,
            secret_phrase_to_account_key,
        },
    },
};
use async_trait::async_trait;
use cardano_multiplatform_lib::{
    address::BaseAddress,
    crypto::PrivateKey,
};
use chacha20::{
    cipher::{
        KeyIvInit,
        StreamCipher,
    },
    ChaCha20,
};
use dialoguer::Password as InputPassword;
use secrecy::{
    ExposeSecret,
    Secret,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::path::PathBuf;
use tokio::fs;

use crate::trireme_ledger_client::cml_client::error::{
    CMLLCError,
    Result as CMLResult,
};

/// Type for representing a password protected phrase
pub struct PasswordProtectedPhraseKeys<P: Password> {
    password: P,
    phrase_file_path: PathBuf,
    encryption_nonce: [u8; 12],
    network: u8,
}

impl<P: Password> PasswordProtectedPhraseKeys<P> {
    /// Constructor for the [`PasswordProtectedPhraseKeys`] struct
    pub fn new(
        password: P,
        phrase_file_path: PathBuf,
        network_index: u8,
        encryption_nonce: [u8; 12],
    ) -> Self {
        Self {
            password,
            phrase_file_path,
            encryption_nonce,
            network: network_index,
        }
    }

    /// Decrypt and read the secret phrase
    async fn read_phrase(&self) -> CMLResult<String> {
        let text = fs::read_to_string(&self.phrase_file_path)
            .await
            .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
        let encrypted: EncryptedSecretPhrase =
            toml::from_str(&text).map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
        let password = self
            .password
            .get_password()
            .map_err(|e| CMLLCError::KeyError(Box::new(e)))?;
        decrypt_phrase(&encrypted, &password, &self.encryption_nonce)
    }
}

#[async_trait]
impl<P: Password + Send + Sync> Keys for PasswordProtectedPhraseKeys<P> {
    async fn base_addr(&self) -> CMLResult<BaseAddress> {
        let phrase = self.read_phrase().await?;
        let account_key = secret_phrase_to_account_key(&phrase)?;
        let base_addr = private_key_to_base_address(&account_key, self.network);
        Ok(base_addr)
    }

    async fn private_key(&self) -> CMLResult<PrivateKey> {
        let phrase = self.read_phrase().await?;
        let account_key = secret_phrase_to_account_key(&phrase)?;
        let priv_key = account_key.derive(0).derive(0).to_raw_key();
        Ok(priv_key)
    }
}

/// Type for holding password
pub struct TerminalPasswordUpfront {
    password: Secret<[u8; 32]>,
}

impl TerminalPasswordUpfront {
    /// Reads the password from the terminal and hashes it with Argon2.
    pub fn init(salt: &[u8]) -> Result<Self> {
        let password = InputPassword::new()
            .with_prompt("Enter password")
            .interact()
            .map_err(|e| Error::Trireme(e.to_string()))?;

        let new = normalize_password(&password, salt)?;
        let secret = Secret::new(new);
        let password_container = Self { password: secret };
        Ok(password_container)
    }
}

/// Normalize the password with Argon2 and specified `salt`
pub fn normalize_password(original: &str, salt: &[u8]) -> Result<[u8; 32]> {
    // TODO: Upgrade config to be more secure?
    let config = argon2::Config::default();
    let hashed = argon2::hash_raw(original.as_bytes(), salt, &config)
        .map_err(|e| Error::Trireme(e.to_string()))?;
    // TODO: Verify that the argon2 output is always long enough
    let new = hashed[..32].try_into().expect("always correct length");
    Ok(new)
}

/// Trait for representing a password
pub trait Password {
    /// Get the password
    fn get_password(&self) -> Result<[u8; 32]>;
}

impl Password for TerminalPasswordUpfront {
    fn get_password(&self) -> Result<[u8; 32]> {
        let password = self.password.expose_secret().to_owned();
        Ok(password)
    }
}

fn decrypt_phrase(
    encrypted_phrase: &EncryptedSecretPhrase,
    password: &[u8; 32],
    encryption_nonce: &[u8; 12],
) -> CMLResult<String> {
    let key = password;
    let mut cipher = ChaCha20::new(key.into(), encryption_nonce.into());
    let mut buffer: Vec<_> = encrypted_phrase.inner.clone();

    cipher.apply_keystream(&mut buffer);

    String::from_utf8(buffer).map_err(|e| CMLLCError::KeyError(Box::new(e)))
}

/// Encrypt a phrase with a password and nonce
pub fn encrypt_phrase(
    phrase: &str,
    password: &[u8; 32],
    encryption_nonce: &[u8; 12],
) -> EncryptedSecretPhrase {
    let key = password;
    let mut cipher = ChaCha20::new(key.into(), encryption_nonce.into());
    let mut buffer: Vec<_> = phrase.bytes().collect();

    cipher.apply_keystream(&mut buffer);

    EncryptedSecretPhrase { inner: buffer }
}

/// Type for holding the encrypted secret phrase
#[derive(Serialize, Deserialize, Debug)]
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
    use tempfile::tempdir;

    struct InMemoryPassword {
        password: [u8; 32],
    }

    impl InMemoryPassword {
        pub fn new(password: [u8; 32]) -> Self {
            Self { password }
        }

        pub fn encrypt_phrase(
            &self,
            phrase: &str,
            encryption_nonce: &[u8; 12],
        ) -> EncryptedSecretPhrase {
            let password = self.get_password().unwrap();
            encrypt_phrase(phrase, &password, encryption_nonce)
        }
    }

    impl Password for InMemoryPassword {
        fn get_password(&self) -> Result<[u8; 32]> {
            Ok(self.password)
        }
    }

    #[tokio::test]
    async fn roundtrip_phrase() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("my-temporary-note.txt");
        let _file = File::create(&file_path).unwrap();

        let original_phrase =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon";

        let nonce = [2; 12];

        let password_raw = "password";
        let password_salt = b"brackish water";

        let normalized = normalize_password(password_raw, password_salt).unwrap();

        let password = InMemoryPassword::new(normalized);

        let encrypted_phrase = password.encrypt_phrase(original_phrase, &nonce);

        write_toml_struct_to_file(&file_path, &encrypted_phrase)
            .await
            .unwrap();

        let keys = PasswordProtectedPhraseKeys::new(password, file_path, 0, nonce);

        let new_phrase = keys.read_phrase().await.unwrap();

        assert_eq!(original_phrase, &new_phrase);
    }

    #[tokio::test]
    async fn decryption_fails_with_wrong_nonce() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("my-temporary-note.txt");
        let _file = File::create(&file_path).unwrap();

        let original_phrase =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
        abandon abandon abandon abandon";

        let true_nonce = [7; 12];
        let false_nonce = [6; 12];

        let password_raw = "password";
        let password_salt = b"brackish water";

        let normalized = normalize_password(password_raw, password_salt).unwrap();

        let password = InMemoryPassword::new(normalized);

        let encrypted_phrase = password.encrypt_phrase(original_phrase, &true_nonce);

        write_toml_struct_to_file(&file_path, &encrypted_phrase)
            .await
            .unwrap();

        let keys = PasswordProtectedPhraseKeys::new(password, file_path, 0, false_nonce);

        let new_phrase = keys.read_phrase().await;

        assert!(new_phrase.is_err());
    }
}
