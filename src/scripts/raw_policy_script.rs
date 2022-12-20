use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::scripts::{as_failed_to_execute, MintingPolicy, ScriptResult, TxContext};
use crate::transaction::TransactionVersion;
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script, PlutusV2Script};
use minicbor::{Decoder, Encoder};
use std::marker::PhantomData;
use uplc::ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term};
use uplc::machine::cost_model::ExBudget;

pub struct RawPolicy<Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _redeemer: PhantomData<Redeemer>,
}

impl<R> RawPolicy<R> {
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let cbor = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v1_policy = RawPolicy {
            version: TransactionVersion::V1,
            cbor: outer.to_vec(),
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }

    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        dbg!(&script_file.cborHex);
        let script_bytes = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v2 = PlutusV2Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v2(&v2);
        let cbor = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = RawPolicy {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }
}

pub struct OneParamRawPolicy<One, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _one: PhantomData<One>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, R> OneParamRawPolicy<One, R> {
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(&script_file.cborHex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamRawPolicy {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _one: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    pub fn apply(&self, one: One) -> RawPlutusScriptResult<RawPolicy<R>> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .unwrap()
                .into();
        let one_data: PlutusData = one.into();
        let one_term = Term::Constant(Constant::Data(one_data.into()));
        // let one_term = Term::Constant(Constant::Integer(69));
        println!("before: {}", program.to_pretty());
        let program = program.apply_term(&one_term);

        let fake: Program<FakeNamedDeBruijn> = program.into();
        println!("after: {}", fake.to_pretty());
        let new_cbor = fake
            .to_cbor()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let cml_script = match self.version {
            TransactionVersion::V1 => {
                todo!()
            }
            TransactionVersion::V2 => {
                let v2 = PlutusV2Script::from_bytes(new_cbor.clone())
                    .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
                PlutusScript::from_v2(&v2)
            }
        };
        let policy = RawPolicy {
            version: self.version.clone(),
            cbor: new_cbor,
            _redeemer: Default::default(),
        };
        Ok(policy)
    }
}

impl<Redeemer> MintingPolicy<Redeemer> for RawPolicy<Redeemer>
where
    Redeemer: Into<PlutusData> + Send + Sync,
{
    fn execute(&self, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<()> {
        dbg!("0");
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .map_err(as_failed_to_execute)?
                .into();
        dbg!("1");
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Constant::Data(redeemer_data.into()));
        let program = program.apply_term(&redeemer_term);
        dbg!("2");
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Constant::Data(ctx_data.into()));
        let program = program.apply_term(&ctx_term);
        dbg!("3");
        let (term, _cost, logs) = match self.version {
            TransactionVersion::V1 => program.eval_v1(),
            TransactionVersion::V2 => program.eval(ExBudget::default()), // TODO: parameterize
        };
        dbg!(&term);
        dbg!(&_cost);
        dbg!(&logs);
        term.map_err(|e| RawPlutusScriptError::AikenEval {
            error: format!("{:?}", e),
            logs,
        })
        .map_err(as_failed_to_execute)?;
        Ok(())
    }

    fn id(&self) -> String {
        let cbor = self.script_hex().unwrap();
        let script = match self.version {
            TransactionVersion::V1 => {
                let script_bytes = hex::decode(&cbor).unwrap(); // TODO: unwrap
                                                                // .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
                let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap(); // TODO: unwrap
                                                                            // .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
                PlutusScript::from_v1(&v1)
            }
            TransactionVersion::V2 => {
                let script_bytes = hex::decode(&cbor).unwrap();
                // .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
                let v2 = PlutusV2Script::from_bytes(script_bytes).unwrap();
                // .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
                PlutusScript::from_v2(&v2)
            }
        };
        script.hash().to_string()
    }

    fn script_hex(&self) -> ScriptResult<String> {
        let wrap = Encoder::new(Vec::new())
            .bytes(&self.cbor)
            .unwrap() // TODO: unwrap
            .clone()
            .into_writer();

        let hex = hex::encode(&wrap);
        Ok(hex)
        // Ok(hex::encode(&self.cbor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT_RAW: &str = include_str!("../../plutus/payment_script.json");

    #[test]
    fn program_stuff() {
        let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW).unwrap();
        let cbor_1 = hex::decode(script_file.cborHex).unwrap();
        println!("original cbor: {}", hex::encode(&cbor_1));
        let mut inner_decoder = Decoder::new(&cbor_1);
        let inner = inner_decoder.bytes().unwrap();
        println!("unwrapped cbor: {}", hex::encode(&inner));
        // let program_1: Program<NamedDeBruijn> =
        let program_1: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&inner, &mut Vec::new())
                .unwrap()
                .into();
        println!("program 1: {}", program_1.to_pretty());
        let one_data: PlutusData = 69i64.into();
        let one_term = Term::Constant(Constant::Data(one_data.into()));
        // let one_term = Term::Constant(Constant::Integer(69));
        let program_applied = program_1.apply_term(&one_term);
        let (t, _, _) = program_applied.eval(Default::default());
        t.unwrap();
        // let fake: Program<FakeNamedDeBruijn> = program_applied.into();
        // let cbor_2 = fake.to_cbor().unwrap();
        // println!("new cbor: {}", hex::encode(&cbor_2));
        // // let wrapped = Encoder::new(Vec::new())
        // //     .bytes(&cbor_2)
        // //     .unwrap() // TODO: unwrap
        // //     .clone()
        // //     .into_writer();
        // // let mut inner_decoder_2 = Decoder::new(&cbor_2);
        // // let inner_2 = inner_decoder_2.bytes().unwrap();
        // // println!("second unwrapped cbor: {}", hex::encode(&inner_2));
        // // let program_2: Program<NamedDeBruijn> =
        // let program_2 = Program::<FakeNamedDeBruijn>::from_cbor(&cbor_2, &mut Vec::new()).unwrap();
        // println!("program 2: {}", program_2.to_pretty());
    }
}
