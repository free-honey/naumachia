use crate::address::shelley_address::{decode_bech32, ShelleyAddress};
use crate::address::shelley_address::{AddressNetwork, ShelleyPaymentPart};
use crate::error::*;
use cardano_multiplatform_lib::address::Address as CMLAddress;
use serde::{Deserialize, Serialize};
use shelley_address::ShelleyDelegationPart;

pub(crate) mod shelley_address;

// TODO: Continue to hone this into a good API. I tried to make the Address generic, but it
//   made for bad ergonomics. Instead, I want to make this as stable as possible.
//   Update: This implicitly wants the string to be a valid addr, but nothing is enforcing that.
//   I don't like that, but I haven't come up with the right design. Creating new issue here:
//   https://github.com/MitchTurner/naumachia/issues/88
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Address {
    Shelley(ShelleyAddress),
    Base(String),
    Script(String),
    Raw(String), // This is a placeholder for now to make tests work
}

impl Address {
    pub fn new(addr: &str) -> Self {
        Address::Raw(addr.to_string())
    }

    pub fn shelley_from_bech32(addr: &str) -> Result<Self> {
        let (_, bytes) = decode_bech32(addr)?;
        bytes_to_address(&bytes)
    }

    pub fn base(addr: &str) -> Self {
        Address::Base(addr.to_string())
    }

    pub fn to_string(&self) -> String {
        match self {
            Address::Base(inner) => inner.to_string(),
            Address::Raw(inner) => inner.to_string(),
            Address::Script(inner) => inner.to_string(),
            Address::Shelley(inner) => inner.to_bech32().unwrap(),
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>> {
        // TODO: I'd rather not take a dep on CML here :(
        let cml_address = CMLAddress::from_bech32(&self.to_string())
            .map_err(|e| Error::Address(format!("Error getting address bytes: {:?}", e)))?;
        Ok(cml_address.to_bytes())
    }
}

fn slice_to_hash(slice: &[u8]) -> Result<[u8; 28]> {
    if slice.len() == 28 {
        let mut sized = [0u8; 28];
        sized.copy_from_slice(slice);
        Ok(sized.into())
    } else {
        Err(Error::Address("Invalid hash size".to_string()))
    }
}

macro_rules! parse_shelley_fn {
    ($name:tt, $payment:tt, pointer) => {
        fn $name(header: u8, payload: &[u8]) -> Result<Address, Error> {
            let net = parse_network(header);
            let h1 = slice_to_hash(&payload[0..=27])?;
            let p1 = ShelleyPaymentPart::$payment(h1);
            let p2 = ShelleyDelegationPart::from_pointer(&payload[28..])?;
            let addr = ShelleyAddress(net, p1, p2);

            Ok(addr.into())
        }
    };
    ($name:tt, $payment:tt, $delegation:tt) => {
        fn $name(header: u8, payload: &[u8]) -> Result<Address, Error> {
            let net = parse_network(header);
            let h1 = slice_to_hash(&payload[0..=27])?;
            let p1 = ShelleyPaymentPart::$payment(h1);
            let h2 = slice_to_hash(&payload[28..=55])?;
            let p2 = ShelleyDelegationPart::$delegation(h2);
            let addr = ShelleyAddress(net, p1, p2);

            Ok(addr.into())
        }
    };
    ($name:tt, $payment:tt) => {
        fn $name(header: u8, payload: &[u8]) -> Result<Address, Error> {
            let net = parse_network(header);
            let h1 = slice_to_hash(&payload[0..=27])?;
            let p1 = ShelleyPaymentPart::$payment(h1);
            let addr = ShelleyAddress(net, p1, ShelleyDelegationPart::Null);

            Ok(addr.into())
        }
    };
}

fn parse_network(header: u8) -> AddressNetwork {
    let masked = header & 0b0000_1111;

    match masked {
        0b_0000_0000 => AddressNetwork::Testnet,
        0b_0000_0001 => AddressNetwork::Mainnet,
        _ => AddressNetwork::Other(masked),
    }
}

// types 0-7 are Shelley addresses
parse_shelley_fn!(parse_type_0, key_hash, key_hash);
parse_shelley_fn!(parse_type_1, script_hash, key_hash);
parse_shelley_fn!(parse_type_2, key_hash, script_hash);
parse_shelley_fn!(parse_type_3, script_hash, script_hash);
parse_shelley_fn!(parse_type_4, key_hash, pointer);
parse_shelley_fn!(parse_type_5, script_hash, pointer);
parse_shelley_fn!(parse_type_6, key_hash);
parse_shelley_fn!(parse_type_7, script_hash);

fn bytes_to_address(bytes: &[u8]) -> Result<Address, Error> {
    let header = *bytes.first().ok_or(Error::MissingHeader)?;
    let payload = &bytes[1..];

    match header & 0b1111_0000 {
        0b0000_0000 => parse_type_0(header, payload),
        0b0001_0000 => parse_type_1(header, payload),
        0b0010_0000 => parse_type_2(header, payload),
        0b0011_0000 => parse_type_3(header, payload),
        0b0100_0000 => parse_type_4(header, payload),
        0b0101_0000 => parse_type_5(header, payload),
        0b0110_0000 => parse_type_6(header, payload),
        0b0111_0000 => parse_type_7(header, payload),
        _ => Err(Error::Address("Invalid Header".to_string())),
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum PolicyId {
    ADA,
    NativeToken(String, Option<String>),
}

impl PolicyId {
    pub fn ada() -> PolicyId {
        PolicyId::ADA
    }

    pub fn native_token(id: &str, asset: &Option<String>) -> PolicyId {
        PolicyId::NativeToken(id.to_string(), asset.to_owned())
    }

    pub fn to_str(&self) -> Option<String> {
        match self {
            PolicyId::ADA => None,
            PolicyId::NativeToken(id, maybe_asset) => {
                if let Some(asset) = maybe_asset {
                    Some(format!("{}-{}", id, asset))
                } else {
                    Some(id.to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cardano_multiplatform_lib::address::Address as CMLAddress;

    #[test]
    fn round_trip_address_bytes() {
        let addr_str = "addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr";
        let address = Address::new(addr_str);
        let bytes = address.bytes().unwrap();
        let new_cml_address = CMLAddress::from_bytes(bytes).unwrap();
        let new_addr_str = new_cml_address
            .to_bech32(Some("addr_test".to_string()))
            .unwrap();
        assert_eq!(addr_str, new_addr_str);
    }
}
