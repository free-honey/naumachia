use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct FakeAddress(String);

impl FakeAddress {
    pub fn new(addr: &str) -> Self {
        FakeAddress(addr.to_string())
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FakeAddress {
    fn from(a_string: String) -> Self {
        FakeAddress(a_string)
    }
}

impl From<FakeAddress> for String {
    fn from(addr: FakeAddress) -> Self {
        addr.to_str().to_owned()
    }
}
