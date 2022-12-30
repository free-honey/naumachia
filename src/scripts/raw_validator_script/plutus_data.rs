use crate::scripts::{TxContext, ValidRange};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PlutusData {
    Constr(Constr<PlutusData>),
    Map(BTreeMap<PlutusData, PlutusData>),
    BigInt(BigInt),
    BoundedBytes(Vec<u8>),
    Array(Vec<PlutusData>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Constr<T> {
    pub tag: u64,
    pub any_constructor: Option<u64>,
    pub fields: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

impl From<BigInt> for i64 {
    fn from(big_int: BigInt) -> Self {
        match big_int {
            BigInt::Int { neg, val } => {
                let value = val as i64;
                if neg {
                    -value
                } else {
                    value
                }
            }
            BigInt::BigUInt(_) => todo!(),
            BigInt::BigNInt(_) => todo!(),
        }
    }
}

impl From<i64> for PlutusData {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        let inner_data = PlutusData::BigInt(BigInt::Int { neg, val });
        // let constr = Constr {
        //     tag: 121,
        //     any_constructor: None,
        //     fields: vec![inner_data],
        // };
        // PlutusData::Constr(constr)
        inner_data
    }
}

// TODO: This is bad!!! Should be a Try
impl From<PlutusData> for i64 {
    fn from(value: PlutusData) -> Self {
        match value {
            PlutusData::BigInt(big_int) => big_int.into(),
            _ => panic!("This isn't a big int"),
        }
    }
}

// TODO: Don't hardcode values!
// TODO: THIS IS V2 only right now! Add V1!
impl From<TxContext> for PlutusData {
    fn from(ctx: TxContext) -> Self {
        let inputs = PlutusData::Array(vec![]);
        let reference_inputs = PlutusData::Array(vec![]);
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
        // let wdrl = PlutusData::Array(vec![]);
        let wdrl = PlutusData::Map(BTreeMap::new());
        let valid_range = ctx.range.into();
        let signatories = PlutusData::Array(vec![]);
        // let redeemers = PlutusData::Array(vec![]);
        let redeemers = PlutusData::Map(BTreeMap::new());
        // let data = PlutusData::Array(vec![]);
        let data = PlutusData::Map(BTreeMap::new());
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
                reference_inputs,
                outputs,
                fee,
                mint,
                dcert,
                wdrl,
                valid_range,
                signatories,
                redeemers,
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

impl From<ValidRange> for PlutusData {
    fn from(value: ValidRange) -> Self {
        match (value.lower, value.upper) {
            (None, None) => no_time_bound(),
            (Some((bound, _)), None) => lower_bound(bound),
            (None, Some(_)) => todo!(),
            (Some(_), Some(_)) => todo!(),
        }
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

fn lower_bound(bound: i64) -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: vec![
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // Finite
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![PlutusData::BigInt(bound.into())],
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

impl From<PlutusData> for () {
    fn from(_: PlutusData) -> Self {}
}
