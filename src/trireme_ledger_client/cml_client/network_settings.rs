use crate::trireme_ledger_client::Network;
use cardano_multiplatform_lib::ledger::common::value::BigNum;

pub struct NetworkSettings {
    network: u8,
    slot_length: i64,
    starting_slot_time: i64,
    starting_slot_number: u64,
}

impl NetworkSettings {
    pub fn new(
        network: u8,
        slot_length: i64,
        starting_slot_time: i64,
        starting_slot_number: u64,
    ) -> Self {
        NetworkSettings {
            network,
            slot_length,
            starting_slot_time,
            starting_slot_number,
        }
    }

    pub fn network(&self) -> u8 {
        self.network
    }

    pub fn slot_length(&self) -> i64 {
        self.slot_length
    }

    pub fn starting_slot_time(&self) -> i64 {
        self.starting_slot_time
    }

    pub fn starting_slot_number(&self) -> u64 {
        self.starting_slot_number
    }

    pub fn slot_from_posix(&self, posix: i64) -> Option<BigNum> {
        let time_s = posix.checked_sub(self.starting_slot_time())?;
        let abs_slot = (time_s / self.slot_length()) as u64 + self.starting_slot_number();
        Some(abs_slot.into())
    }
}

const MAINNET_NETWORK: u8 = 1;
const MAINNET_SLOT_LENGTH: i64 = 1;
const MAINNET_STARTING_SLOT_TIME: i64 = 1596059091;
const MAINNET_STARTING_SLOT_NUMBER: u64 = 4492800;

const PRE_PROD_NETWORK: u8 = 0;
const PRE_PROD_SLOT_LENGTH: i64 = 1;
const PRE_PROD_STARTING_SLOT_TIME: i64 = 1655769600;
const PRE_PROD_STARTING_SLOT_NUMBER: u64 = 86400;

const PREVIEW_NETWORK: u8 = 0;
const PREVIEW_SLOT_LENGTH: i64 = 1;
const PREVIEW_STARTING_SLOT_TIME: i64 = 1666656000;
const PREVIEW_STARTING_SLOT_NUMBER: u64 = 0;

impl From<Network> for NetworkSettings {
    fn from(network: Network) -> Self {
        match network {
            Network::Preprod => NetworkSettings::new(
                PRE_PROD_NETWORK,
                PRE_PROD_SLOT_LENGTH,
                PRE_PROD_STARTING_SLOT_TIME,
                PRE_PROD_STARTING_SLOT_NUMBER,
            ),
            Network::Mainnet => NetworkSettings::new(
                MAINNET_NETWORK,
                MAINNET_SLOT_LENGTH,
                MAINNET_STARTING_SLOT_TIME,
                MAINNET_STARTING_SLOT_NUMBER,
            ),
            Network::Preview => NetworkSettings::new(
                PREVIEW_NETWORK,
                PREVIEW_SLOT_LENGTH,
                PREVIEW_STARTING_SLOT_TIME,
                PREVIEW_STARTING_SLOT_NUMBER,
            ),
            Network::Testnet => {
                unimplemented!("Testnet is no longer supported")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn slot_from_posix__mainnet() {
        // given
        let network = Network::Mainnet;
        let network_settings = NetworkSettings::from(network);

        // when
        let posix = 1693686614;

        // then
        let expected: BigNum = 102120323u64.into();
        let actual = network_settings.slot_from_posix(posix).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn slot_from_posix__preprod() {
        // given
        let network = Network::Preprod;
        let network_settings = NetworkSettings::from(network);

        // when
        let posix = 1693686777;

        // then
        let expected: BigNum = 38003577u64.into();
        let actual = network_settings.slot_from_posix(posix).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn slot_from_posix__preview() {
        // given
        let network = Network::Preview;
        let network_settings = NetworkSettings::from(network);

        // when
        let posix = 1693686823;

        // then
        let expected: BigNum = 27030823u64.into();
        let actual = network_settings.slot_from_posix(posix).unwrap();
        assert_eq!(expected, actual);
    }
}