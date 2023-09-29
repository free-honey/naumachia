use serde::{Deserialize, Serialize};

/// Token identity.
#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum PolicyId {
    /// ADA as denominated in Lovelace (1 ADA = 1_000_000 Lovelace)
    Lovelace,
    /// Native token with policy id and optional asset name
    NativeToken(String, Option<String>),
}

impl PolicyId {
    /// Constructor for Lovelace policy id
    pub fn ada() -> PolicyId {
        PolicyId::Lovelace
    }

    /// Constructor for native token policy id
    pub fn native_token(id: &str, asset: &Option<String>) -> PolicyId {
        PolicyId::NativeToken(id.to_string(), asset.to_owned())
    }

    /// Getter for policy id
    pub fn id(&self) -> String {
        match self {
            PolicyId::Lovelace => "".to_string(),
            PolicyId::NativeToken(id, _) => id.clone(),
        }
    }

    /// Getter for asset name
    pub fn asset_name(&self) -> Option<String> {
        match self {
            PolicyId::Lovelace => None,
            PolicyId::NativeToken(_, asset_name) => asset_name.to_owned(),
        }
    }
}
