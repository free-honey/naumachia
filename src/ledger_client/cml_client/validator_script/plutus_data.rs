use crate::scripts::TxContext;
use std::collections::BTreeMap;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PlutusData {
    Constr(Constr<PlutusData>),
    Map(BTreeMap<PlutusData, PlutusData>),
    BigInt(BigInt),
    BoundedBytes(Vec<u8>),
    Array(Vec<PlutusData>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Constr<T> {
    pub tag: u64,
    pub any_constructor: Option<u64>,
    pub fields: Vec<T>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum BigInt {
    Int { neg: bool, val: u64 },
    BigUInt(Vec<u8>),
    BigNInt(Vec<u8>),
}

impl From<i64> for BigInt {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        BigInt::Int { neg, val }
    }
}

impl From<i64> for PlutusData {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        let inner_data = PlutusData::BigInt(BigInt::Int { neg, val });
        let constr = Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![inner_data],
        };
        PlutusData::Constr(constr)
    }
}

// TODO: Don't hardcode values!
// TODO: Cover V2 as well (this is V1)
impl From<TxContext> for PlutusData {
    fn from(_: TxContext) -> Self {
        let inputs = PlutusData::Array(vec![]);
        let outputs = PlutusData::Array(vec![]);
        let fee = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(999_i64.into()),
            )])),
        )]));
        let mint = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(0_i64.into()),
            )])),
        )]));
        let dcert = PlutusData::Array(vec![]);
        let wdrl = PlutusData::Array(vec![]);
        let valid_range = no_time_bound();
        let signatories = PlutusData::Array(vec![]);
        let data = PlutusData::Array(vec![]);
        let id = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(Vec::new())],
        });
        let tx_info = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                inputs,
                outputs,
                fee,
                mint,
                dcert,
                wdrl,
                valid_range,
                signatories,
                data,
                id,
            ],
        });
        // Spending
        let purpose = PlutusData::Constr(Constr {
            tag: 122,
            any_constructor: None,
            fields: vec![],
        });
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![tx_info, purpose],
        })
    }
}

fn no_time_bound() -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: vec![
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // NegInf
                    PlutusData::Constr(Constr {
                        tag: 121,
                        any_constructor: None,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // PosInf
                    PlutusData::Constr(Constr {
                        tag: 123,
                        any_constructor: None,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
        ],
    })
}

impl From<()> for PlutusData {
    fn from(_: ()) -> Self {
        PlutusData::BoundedBytes(Vec::new())
    }
}
