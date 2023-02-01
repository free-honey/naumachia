use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ShelleyAddress {
    pub network: AddressNetwork,
    pub shelley_payment_part: ShelleyPaymentPart,
    pub shelley_delegation_part: ShelleyDelegationPart,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AddressNetwork {
    Testnet,
    Mainnet,
    Other(u8),
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

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ShelleyDelegationPart {
    Key([u8; 28]),
    Script([u8; 28]),
    Pointer(Pointer),
    Null,
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
