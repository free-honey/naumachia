use naumachia::scripts::{ScriptError, ScriptResult};
use naumachia::trireme_ledger_client::cml_client::validator_script::plutus_data::PlutusData;
use naumachia::trireme_ledger_client::cml_client::validator_script::{
    PlutusScriptFile, RawPlutusValidator,
};
use sha2::Digest;
use sha2::Sha256;

const SCRIPT_RAW: &str = include_str!("../../plutus/game.plutus");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HashedString {
    inner: Vec<u8>,
}

impl HashedString {
    pub fn new(unhashed: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(unhashed.as_bytes());
        let inner = hasher.finalize().to_vec();
        HashedString { inner }
    }
}

impl From<HashedString> for PlutusData {
    fn from(hs: HashedString) -> Self {
        let bytes = hs.inner;
        PlutusData::BoundedBytes(bytes)
    }
}

impl TryFrom<PlutusData> for HashedString {
    type Error = ScriptError;

    fn try_from(data: PlutusData) -> Result<Self, Self::Error> {
        match data {
            PlutusData::BoundedBytes(inner) => Ok(HashedString { inner }),
            _ => Err(ScriptError::DatumDeserialization(format!("{:?}", data))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClearString {
    inner: String,
}

impl ClearString {
    pub fn new(inner: &str) -> Self {
        ClearString {
            inner: inner.to_string(),
        }
    }
}

impl From<ClearString> for PlutusData {
    fn from(cs: ClearString) -> Self {
        let bytes = cs.inner.as_bytes().to_vec();
        PlutusData::BoundedBytes(bytes)
    }
}

impl TryFrom<PlutusData> for ClearString {
    type Error = ScriptError;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::BoundedBytes(ref bytes) => {
                let inner = String::from_utf8(bytes.clone())
                    .map_err(|_| ScriptError::RedeemerDeserialization(format!("{:?}", value)))?;
                Ok(ClearString { inner })
            }
            _ => Err(ScriptError::RedeemerDeserialization(format!("{:?}", value))),
        }
    }
}

pub fn get_script() -> ScriptResult<RawPlutusValidator<HashedString, ClearString>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = RawPlutusValidator::new_v1(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}