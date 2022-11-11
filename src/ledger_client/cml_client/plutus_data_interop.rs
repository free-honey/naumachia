use crate::ledger_client::cml_client::validator_script::plutus_data::{BigInt, Constr, PlutusData};
use cardano_multiplatform_lib::ledger::common::value::BigInt as CMLBigInt;
use cardano_multiplatform_lib::plutus::{
    ConstrPlutusData as CMLConstrPlutusData, PlutusData as CMLPlutusData,
    PlutusList as CMLPlutusList, PlutusMap as CMLPlutusMap,
};

pub enum PlutusDataInteropError {
    CannotDeserializeFromPlutusData(PlutusData),
}

pub type Result<T, E = PlutusDataInteropError> = std::result::Result<T, E>;

pub trait PlutusDataInterop: Sized {
    type Error;
    fn to_plutus_data(&self) -> CMLPlutusData;
    fn from_plutus_data(plutus_data: &CMLPlutusData) -> Result<Self, Self::Error>;
}

impl<T> PlutusDataInterop for T
where
    T: Into<PlutusData> + TryFrom<PlutusData> + Clone,
{
    type Error = <T as TryFrom<PlutusData>>::Error;

    fn to_plutus_data(&self) -> CMLPlutusData {
        let nau_data: PlutusData = self.to_owned().into();
        nau_data.into()
    }

    fn from_plutus_data(plutus_data: &CMLPlutusData) -> Result<Self, Self::Error> {
        let nau_data: PlutusData = plutus_data.clone().into();
        nau_data.try_into()
    }
}

// TODO: LEt's get some prop tests up in here
impl From<PlutusData> for CMLPlutusData {
    fn from(data: PlutusData) -> Self {
        match data {
            PlutusData::Constr(constr) => CMLPlutusData::new_constr_plutus_data(&constr.into()),
            PlutusData::Map(map) => {
                let mut plutus_map = CMLPlutusMap::new();
                map.into_iter()
                    .map(|(key, value)| (CMLPlutusData::from(key), CMLPlutusData::from(value)))
                    .for_each(|(key, value)| {
                        plutus_map.insert(&key, &value);
                    });
                CMLPlutusData::new_map(&plutus_map)
            }
            PlutusData::BigInt(big_int) => CMLPlutusData::new_integer(&big_int.into()),
            PlutusData::BoundedBytes(_) => {
                let bytes = todo!();
                CMLPlutusData::new_bytes(bytes)
            }
            PlutusData::Array(_) => {
                let list = todo!();
                CMLPlutusData::new_list(&list)
            }
        }
    }
}

impl From<Constr<PlutusData>> for CMLConstrPlutusData {
    fn from(constr: Constr<PlutusData>) -> Self {
        let mut data = CMLPlutusList::new();
        constr.fields.into_iter().for_each(|d| data.add(&d.into()));
        CMLConstrPlutusData::new(&constr.tag.into(), &data)
    }
}

impl From<BigInt> for CMLBigInt {
    fn from(big_int: BigInt) -> Self {
        match big_int {
            BigInt::Int { neg, val } => {
                let mut string = match neg {
                    true => "-".to_string(),
                    false => "".to_string(),
                };
                string.push_str(&val.to_string());
                // TODO: unwrap
                CMLBigInt::from_str(&string).expect("Can this fail?")
            }
            BigInt::BigUInt(_) => todo!(),
            BigInt::BigNInt(_) => todo!(),
        }
    }
}

impl From<CMLBigInt> for BigInt {
    fn from(value: CMLBigInt) -> Self {
        let string = value.to_str();
        dbg!(&string);
        todo!()
    }
}

impl From<CMLPlutusData> for PlutusData {
    fn from(_: CMLPlutusData) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_big_int() {
        let original: i64 = -1438924;
        let nau_big_int = BigInt::from(original);
        let cml_big_int = CMLBigInt::from(nau_big_int);
        let new_nau_big_int = BigInt::from(cml_big_int);
        let new: i64 = new_nau_big_int.into();
        assert_eq!(original, new);
    }
}
