use crate::scripts::raw_script::{PlutusScriptFile, RawPlutusScriptError, RawPlutusScriptResult};
use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::scripts::{as_failed_to_execute, MintingPolicy, ScriptResult, TxContext};
use crate::transaction::ScriptVersion;
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script, PlutusV2Script};
use minicbor::Decoder;
use std::marker::PhantomData;
use uplc::ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term};

pub struct RawPolicy<Redeemer> {
    cbor_hex: String,
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
        let v1_policy = RawPolicy {
            cbor_hex: script_file.cborHex,
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
        let v1_policy = RawPolicy {
            cbor_hex: script_file.cborHex,
            cml_script,
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }
}

pub struct OneParamRawPolicy<One, Redeemer> {
    version: ScriptVersion,
    cbor_hex: String,
    _one: PhantomData<One>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, R> OneParamRawPolicy<One, R> {
    fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let v2_val = OneParamRawPolicy {
            version: ScriptVersion::V2,
            cbor_hex: script_file.cborHex,
            _one: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    fn apply(&self, one: One) -> RawPlutusScriptResult<RawPolicy<R>> {
        let cbor = hex::decode(&self.cbor_hex)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let mut flat_decoder = Decoder::new(outer);
        let flat = flat_decoder
            .bytes()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let program: Program<NamedDeBruijn> = Program::<FakeNamedDeBruijn>::from_flat(flat)
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?
            .into();
        let one_data: PlutusData = one.into();
        let datum_term = Term::Constant(Constant::Data(one_data.into()));
        let program = program.apply_term(&datum_term);
        let new_cbor = program
            .to_cbor()
            .map_err(|e| RawPlutusScriptError::AikenApply(e.to_string()))?;
        let cbor_hex = hex::encode(new_cbor);
        let script_bytes =
            hex::decode(&cbor_hex).map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let v2 = PlutusV2Script::from_bytes(script_bytes)
            .map_err(|e| RawPlutusScriptError::CMLError(e.to_string()))?;
        let cml_script = PlutusScript::from_v2(&v2);
        let policy = RawPolicy {
            cbor_hex,
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
    fn execute(&self, _redeemer: Redeemer, _ctx: TxContext) -> ScriptResult<()> {
        todo!()
    }

    fn id(&self) -> String {
        let script_hash = self.cml_script.hash();
        script_hash.to_string()
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        Ok(&self.cbor_hex)
    }
}
