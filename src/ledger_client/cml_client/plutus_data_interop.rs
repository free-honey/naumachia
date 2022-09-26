use cardano_multiplatform_lib::plutus::PlutusData;

pub enum PlutusDataInteropError {
    CannotDeserializeFromPlutusData(PlutusData),
}

pub type Result<T, E = PlutusDataInteropError> = std::result::Result<T, E>;

pub trait PlutusDataInterop: Sized {
    fn to_plutus_data(&self) -> PlutusData;
    fn from_plutus_data(plutus_data: &PlutusData) -> Result<Self>;
}

// TODO: IDK if this is right
impl PlutusDataInterop for () {
    fn to_plutus_data(&self) -> PlutusData {
        PlutusData::new_bytes(Vec::new())
    }

    fn from_plutus_data(_plutus_data: &PlutusData) -> Result<Self> {
        Ok(())
    }
}
