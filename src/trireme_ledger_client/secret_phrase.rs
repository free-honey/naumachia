use crate::trireme_ledger_client::{
    cml_client::error::{CMLLCError, Result as CMLLCResult},
    raw_secret_phrase::RawSecretPhraseKeysError,
};
use bip39::{Language, Mnemonic};
use cardano_multiplatform_lib::{
    address::{BaseAddress, StakeCredential},
    crypto::Bip32PrivateKey,
};

pub fn secret_phrase_to_account_key(phrase: &str) -> CMLLCResult<Bip32PrivateKey> {
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

pub fn private_key_to_base_address(account_key: &Bip32PrivateKey, network: u8) -> BaseAddress {
    let pub_key = account_key.derive(0).derive(0).to_public();
    let stake_key = account_key.derive(2).derive(0).to_public();
    let pub_key_creds = StakeCredential::from_keyhash(&pub_key.to_raw_key().hash());
    let stake_key_creds = StakeCredential::from_keyhash(&stake_key.to_raw_key().hash());
    let base_addr = BaseAddress::new(network, &pub_key_creds, &stake_key_creds);
    base_addr
}

fn harden(index: u32) -> u32 {
    index | 0x80_00_00_00
}
