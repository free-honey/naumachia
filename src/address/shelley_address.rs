use crate::error::*;
use cardano_multiplatform_lib::chain_crypto::bech32;
use minicbor::encode;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ShelleyAddress {
    pub network: AddressNetwork,
    pub shelley_payment_part: ShelleyPaymentPart,
    pub shelley_delegation_part: ShelleyDelegationPart,
}

impl ShelleyAddress {
    pub fn new(
        network: AddressNetwork,
        shelley_payment_part: ShelleyPaymentPart,
        shelley_delegation_part: ShelleyDelegationPart,
    ) -> Self {
        ShelleyAddress {
            network,
            shelley_payment_part,
            shelley_delegation_part,
        }
    }

    /// Gets the network assoaciated with this address
    pub fn network(&self) -> AddressNetwork {
        self.network.clone()
    }

    /// Gets a numeric id describing the type of the address
    pub fn typeid(&self) -> u8 {
        match (&self.shelley_payment_part, &self.shelley_delegation_part) {
            (ShelleyPaymentPart::Key(_), ShelleyDelegationPart::Key(_)) => 0b0000,
            (ShelleyPaymentPart::Script(_), ShelleyDelegationPart::Key(_)) => 0b0001,
            (ShelleyPaymentPart::Key(_), ShelleyDelegationPart::Script(_)) => 0b0010,
            (ShelleyPaymentPart::Script(_), ShelleyDelegationPart::Script(_)) => 0b0011,
            (ShelleyPaymentPart::Key(_), ShelleyDelegationPart::Pointer(_)) => 0b0100,
            (ShelleyPaymentPart::Script(_), ShelleyDelegationPart::Pointer(_)) => 0b0101,
            (ShelleyPaymentPart::Key(_), ShelleyDelegationPart::Null) => 0b0110,
            (ShelleyPaymentPart::Script(_), ShelleyDelegationPart::Null) => 0b0111,
        }
    }

    pub fn to_header(&self) -> u8 {
        let type_id = self.typeid();
        let type_id = type_id << 4;
        let network = self.network.value();

        type_id | network
    }

    pub fn payment(&self) -> &ShelleyPaymentPart {
        &self.shelley_payment_part
    }

    pub fn delegation(&self) -> &ShelleyDelegationPart {
        &self.shelley_delegation_part
    }

    /// Gets the bech32 human-readable-part for this address
    pub fn hrp(&self) -> Result<&'static str> {
        match &self.network {
            AddressNetwork::Testnet => Ok("addr_test"),
            AddressNetwork::Mainnet => Ok("addr"),
            AddressNetwork::Other(x) => Err(Error::Address(format!(
                "Can't construct human readable part of address from value: {:?}",
                x
            ))),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let header = self.to_header();
        let payment = self.shelley_payment_part.bytes();
        let delegation = self.shelley_delegation_part.bytes();

        [&[header], payment.as_slice(), delegation.as_slice()].concat()
    }

    pub fn to_hex(&self) -> String {
        let bytes = self.to_vec();
        hex::encode(bytes)
    }

    pub fn to_bech32(&self) -> Result<String> {
        let hrp = self.hrp()?;
        let bytes = self.to_vec();
        encode_bech32(&bytes, hrp)
    }
}

fn encode_bech32(addr: &[u8], hrp: &str) -> Result<String> {
    let base32 = ::bech32::ToBase32::to_base32(&addr);
    ::bech32::encode(hrp, base32, ::bech32::Variant::Bech32)
        .map_err(|e| Error::Address(format!("Bad Bech32: {:?}", e)))
}

pub fn decode_bech32(bech32: &str) -> Result<(String, Vec<u8>), Error> {
    let (hrp, addr, _) =
        ::bech32::decode(bech32).map_err(|e| Error::Address(format!("Bad Bech32: {:?}", e)))?;
    let base10 = ::bech32::FromBase32::from_base32(&addr)
        .map_err(|e| Error::Address(format!("Bad Bech32: {:?}", e)))?;
    Ok((hrp, base10))
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, Copy)]
pub enum AddressNetwork {
    Testnet,
    Mainnet,
    Other(u8),
}

impl AddressNetwork {
    pub fn value(&self) -> u8 {
        match self {
            AddressNetwork::Testnet => 0,
            AddressNetwork::Mainnet => 1,
            AddressNetwork::Other(x) => *x,
        }
    }
}

impl From<u8> for AddressNetwork {
    fn from(id: u8) -> Self {
        match id {
            0 => AddressNetwork::Testnet,
            1 => AddressNetwork::Mainnet,
            x => AddressNetwork::Other(x),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ShelleyPaymentPart {
    Key([u8; 28]),
    Script([u8; 28]),
}

impl ShelleyPaymentPart {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            ShelleyPaymentPart::Key(inner) => inner.to_vec(),
            ShelleyPaymentPart::Script(inner) => inner.to_vec(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ShelleyDelegationPart {
    Key([u8; 28]),
    Script([u8; 28]),
    Pointer(Pointer),
    Null,
}

impl ShelleyDelegationPart {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            ShelleyDelegationPart::Key(inner) => inner.to_vec(),
            ShelleyDelegationPart::Script(inner) => inner.to_vec(),
            ShelleyDelegationPart::Pointer(inner) => {
                todo!()
            }
            ShelleyDelegationPart::Null => {
                vec![]
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Pointer(u64, u64, u64);

impl Pointer {
    pub fn slot(&self) -> u64 {
        self.0
    }

    pub fn tx_idx(&self) -> u64 {
        self.1
    }

    pub fn cert_idx(&self) -> u64 {
        self.2
    }
}
