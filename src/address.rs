use serde::{Deserialize, Serialize};

// TODO: Continue to hone this into a good API. I tried to make the Address generic, but it
//   made for bad ergonomics. Instead, I want to make this as stable as possible.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Address {
    Base(String),
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
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum PolicyId {
    ADA,
    NativeToken(String),
}

impl PolicyId {
    pub fn ada() -> PolicyId {
        PolicyId::ADA
    }

    pub fn native_token(id: &str) -> PolicyId {
        PolicyId::NativeToken(id.to_string())
    }
}
