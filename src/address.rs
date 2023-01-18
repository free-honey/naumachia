use crate::error::*;
use cardano_multiplatform_lib::address::Address as CMLAddress;
use serde::{Deserialize, Serialize};

// TODO: Continue to hone this into a good API. I tried to make the Address generic, but it
//   made for bad ergonomics. Instead, I want to make this as stable as possible.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Address {
    Base(String),
    Script(String),
    Raw(String), // This is a placeholder for now to make tests work
}

impl Address {
    pub fn new(addr: &str) -> Self {
        Address::Raw(addr.to_string())
    }

    pub fn base(addr: &str) -> Self {
        Address::Base(addr.to_string())
    }

    pub fn to_str(&self) -> &str {
        match self {
            Address::Base(inner) => inner,
            Address::Raw(inner) => inner,
            Address::Script(inner) => inner,
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>> {
        // TODO: I'd rather not take a dep on CML here :(
        let cml_address = CMLAddress::from_bech32(self.to_str())
            .map_err(|e| Error::Address(format!("Error getting address bytes: {:?}", e)))?;
        Ok(cml_address.to_bytes())
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
