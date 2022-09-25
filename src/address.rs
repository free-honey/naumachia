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
