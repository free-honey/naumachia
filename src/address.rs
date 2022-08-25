use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

pub trait ValidAddress:
    PartialEq + Eq + Hash + Clone + Debug + Serialize + DeserializeOwned + From<String> + Into<String>
{
}

impl<T> ValidAddress for T where
    T: PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + From<String>
        + Into<String>
{
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
