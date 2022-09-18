use crate::CONFIG_PATH;
use bip39::{Language, Mnemonic};
use cardano_multiplatform_lib::address::{BaseAddress, StakeCredential};
use cardano_multiplatform_lib::crypto::{Bip32PrivateKey, PrivateKey};
use std::fs;
use std::path::Path;

pub(crate) const TESTNET: u8 = 0;

pub fn my_base_addr() -> BaseAddress {
    let phrase = load_phrase_from_file(CONFIG_PATH);
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

    let entropy = mnemonic.entropy();

    base_address_from_entropy(entropy, TESTNET)
}

// TODO: Dedupe
pub fn my_priv_key() -> PrivateKey {
    let phrase = load_phrase_from_file(CONFIG_PATH);
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

    let entropy = mnemonic.entropy();
    let root_key = Bip32PrivateKey::from_bip39_entropy(entropy, &[]);

    let account_key = root_key
        .derive(harden(1852))
        .derive(harden(1815))
        .derive(harden(0));

    account_key.derive(0).derive(0).to_raw_key()
}

fn harden(index: u32) -> u32 {
    index | 0x80_00_00_00
}

fn base_address_from_entropy(entropy: &[u8], network: u8) -> BaseAddress {
    let root_key = Bip32PrivateKey::from_bip39_entropy(entropy, &[]);

    let account_key = root_key
        .derive(harden(1852))
        .derive(harden(1815))
        .derive(harden(0));

    let pub_key = account_key.derive(0).derive(0).to_public();
    let stake_key = account_key.derive(2).derive(0).to_public();

    let pub_key_creds = StakeCredential::from_keyhash(&pub_key.to_raw_key().hash());
    let stake_key_creds = StakeCredential::from_keyhash(&stake_key.to_raw_key().hash());

    BaseAddress::new(network, &pub_key_creds, &stake_key_creds)
}

fn load_phrase_from_file(config_path: &str) -> String {
    let path = Path::new(config_path);
    let text = fs::read_to_string(&path).unwrap();
    let config: toml::Value = toml::from_str(&text).unwrap();
    config["phrase"].as_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::{Language, Mnemonic};

    // Must include a TOML file at your project root with the field:
    //   phrase = <INSERT SECRET PHRASE HERE>
    const CONFIG_PATH: &str = ".blockfrost.toml";

    #[ignore]
    #[test]
    fn learn_root() {
        let phrase = load_phrase_from_file(CONFIG_PATH);
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

        let entropy = mnemonic.entropy();

        let base_addr = base_address_from_entropy(entropy, TESTNET);

        dbg!(base_addr.to_address().to_bech32(None).unwrap());
    }
}
