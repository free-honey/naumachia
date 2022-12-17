use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::scripts::{as_failed_to_execute, MintingPolicy, ScriptResult, TxContext};
use crate::transaction::TransactionVersion;
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script, PlutusV2Script};
use minicbor::{Decoder, Encoder};
use std::marker::PhantomData;
use uplc::ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term};

pub struct RawPolicy<Redeemer> {
    cbor: Vec<u8>,
    cml_script: PlutusScript,
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
            cbor: outer.to_vec(),
            cml_script,
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }

    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
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
        let v1_policy = RawPolicy {
            cbor: outer.to_vec(),
            cml_script,
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
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
        let datum_term = Term::Constant(Constant::Data(one_data.into()));
        let program = program.apply_term(&datum_term);
        let new_cbor = program
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
            cbor: new_cbor,
            cml_script,
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
        let program =
            Program::from_cbor(&self.cbor, &mut Vec::new()).map_err(as_failed_to_execute)?;
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Constant::Data(redeemer_data.into()));
        let program = program.apply_term(&redeemer_term);
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Constant::Data(ctx_data.into()));
        let program = program.apply_term(&ctx_term);
        let (term, _cost, logs) = program.eval_v1();
        term.map_err(|e| RawPlutusScriptError::AikenEval {
            error: format!("{:?}", e),
            logs,
        })
        .map_err(as_failed_to_execute)?;
        Ok(())
    }

    fn id(&self) -> String {
        let script_hash = self.cml_script.hash();
        script_hash.to_string()
    }

    fn script_hex(&self) -> ScriptResult<String> {
        let hex = hex::encode(&self.cbor);
        Ok(hex)
    }
}
