use crate::trireme_ledger_client::Network;

pub struct NetworkSettings {
    network: u8,
    slot_length: i64,
    first_slot_time: i64,
}

impl NetworkSettings {
    pub fn new(network: u8, slot_length: i64, first_slot_time: i64) -> Self {
        NetworkSettings {
            network,
            slot_length,
            first_slot_time,
        }
    }

    pub fn network(&self) -> u8 {
        self.network
    }

    pub fn slot_length(&self) -> i64 {
        self.slot_length
    }

    pub fn first_slot_time(&self) -> i64 {
        self.first_slot_time
    }
}

const MAINNET_NETWORK: u8 = 1;
const MAINNET_SLOT_LENGTH: i64 = 1;
const MAINNET_FIRST_SLOT_TIME: i64 = 1596059091;

const PRE_PROD_NETWORK: u8 = 0;
const PRE_PROD_SLOT_LENGTH: i64 = 1;
const PRE_PROD_FIRST_SLOT_TIME: i64 = 1654041600;

const PREVIEW_NETWORK: u8 = 0;
const PREVIEW_SLOT_LENGTH: i64 = 1;
const PREVIEW_FIRST_SLOT_TIME: i64 = 1660003200;

const TESTNET_NETWORK: u8 = 0;
const TESTNET_SLOT_LENGTH: i64 = 1;
const TESTNET_FIRST_SLOT_TIME: i64 = 1595967616;

impl From<Network> for NetworkSettings {
    fn from(network: Network) -> Self {
        match network {
            Network::Preprod => NetworkSettings::new(
                PRE_PROD_NETWORK,
                PRE_PROD_SLOT_LENGTH,
                PRE_PROD_FIRST_SLOT_TIME,
            ),
            Network::Mainnet => NetworkSettings::new(
                MAINNET_NETWORK,
                MAINNET_SLOT_LENGTH,
                MAINNET_FIRST_SLOT_TIME,
            ),
            Network::Preview => NetworkSettings::new(
                PREVIEW_NETWORK,
                PREVIEW_SLOT_LENGTH,
                PREVIEW_FIRST_SLOT_TIME,
            ),
            Network::Testnet => NetworkSettings::new(
                TESTNET_NETWORK,
                TESTNET_SLOT_LENGTH,
                TESTNET_FIRST_SLOT_TIME,
            ),
        }
    }
}
