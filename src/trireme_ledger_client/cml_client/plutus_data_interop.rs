use crate::scripts::raw_validator_script::plutus_data::{BigInt, Constr, PlutusData};
use cardano_multiplatform_lib::{
    ledger::common::value::BigInt as CMLBigInt,
    plutus::{
        ConstrPlutusData as CMLConstrPlutusData, PlutusData as CMLPlutusData, PlutusDataKind,
        PlutusList as CMLPlutusList, PlutusList, PlutusMap as CMLPlutusMap,
    },
};
use std::collections::BTreeMap;

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
            PlutusData::BoundedBytes(bytes) => CMLPlutusData::new_bytes(bytes),
            PlutusData::Array(array) => {
                let mut list = PlutusList::new();
                array.into_iter().for_each(|data| list.add(&data.into()));
                CMLPlutusData::new_list(&list)
            }
        }
    }
}

impl From<Constr<PlutusData>> for CMLConstrPlutusData {
    fn from(constr: Constr<PlutusData>) -> Self {
        let mut data = CMLPlutusList::new();
        constr.fields.into_iter().for_each(|d| data.add(&d.into()));
        CMLConstrPlutusData::new(&constr.constr.into(), &data)
    }
}

impl From<CMLConstrPlutusData> for Constr<PlutusData> {
    fn from(constr: CMLConstrPlutusData) -> Self {
        let tag = constr.alternative().into();
        let mut fields = Vec::new();
        let data = constr.data();
        let len = data.len();
        for i in 0..len {
            fields.push(data.get(i).into());
        }
        Constr {
            constr: tag,
            fields,
        }
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
                CMLBigInt::from_str(&string).expect("Can this fail?") // TODO: unwrap
            }
            BigInt::BigUInt(_) => todo!(),
            BigInt::BigNInt(_) => todo!(),
        }
    }
}

impl From<CMLBigInt> for BigInt {
    fn from(value: CMLBigInt) -> Self {
        let string = value.to_str();
        let num = string.parse::<i64>().unwrap(); // TODO: unwrap
        num.into()
    }
}

impl From<CMLPlutusData> for PlutusData {
    fn from(value: CMLPlutusData) -> Self {
        match value.kind() {
            PlutusDataKind::ConstrPlutusData => {
                let constr = value.as_constr_plutus_data().expect("Should be a constr");
                PlutusData::Constr(constr.into())
            }
            PlutusDataKind::Map => {
                let map = value.as_map().expect("Should be a map");
                let keys = map.keys();
                let mut keys_vec = Vec::new();
                let keys_len = map.len();
                for i in 0..keys_len {
                    let key = keys.get(i);
                    keys_vec.push(key);
                }
                let mut inner = BTreeMap::new();
                keys_vec.iter().for_each(|key| {
                    inner.insert(
                        key.clone().into(),
                        map.get(key).expect("should exist").into(),
                    );
                });
                PlutusData::Map(inner)
            }
            PlutusDataKind::List => {
                let list = value.as_list().expect("Should be a list");
                let len = list.len();
                let mut array = Vec::new();
                for i in 0..len {
                    array.push(list.get(i).into())
                }
                PlutusData::Array(array)
            }
            PlutusDataKind::Integer => {
                let int = value.as_integer().expect("Should be a int");
                PlutusData::BigInt(int.into())
            }
            PlutusDataKind::Bytes => {
                let bytes = value.as_bytes().expect("Should be bytes");
                PlutusData::BoundedBytes(bytes)
            }
        }
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
