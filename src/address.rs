use serde::{Deserialize, Serialize};

// TODO: As of now, wallet, script, and policy addresses are the same. This is an
//       over-simplification in many ways. Wallet and address also need to be disambiguated.
//       Really, it should be an assoc type for your TxORecord impl because we don't care what
//       it is in our domain, as long as it's unique, and comparable.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct Address(String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

// TODO: This should represent PolicyId as well as AssetName. Maybe a custom enum would be good
pub type PolicyId = Option<Address>;

pub const ADA: PolicyId = None;
