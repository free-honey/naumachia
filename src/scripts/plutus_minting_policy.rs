use crate::scripts::ExecutionCost;
use crate::{
    scripts::context::TxContext,
    scripts::raw_script::ValidatorBlueprint,
    scripts::ScriptError,
    scripts::{
        as_failed_to_execute,
        plutus_validator::plutus_data::PlutusData,
        raw_script::{PlutusScriptError, PlutusScriptFile, RawPlutusScriptResult},
        MintingPolicy, ScriptResult,
    },
    transaction::TransactionVersion,
};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script, PlutusV2Script};
use minicbor::{Decoder, Encoder};
use pallas_primitives::babbage::Language;
use std::marker::PhantomData;
use std::rc::Rc;
use uplc::{
    ast::{Constant, FakeNamedDeBruijn, NamedDeBruijn, Program, Term},
    machine::cost_model::ExBudget,
};

/// Implementation of [`MintingPolicy`] for UPLC Minting Policies
pub struct PlutusMintingPolicy<Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _redeemer: PhantomData<Redeemer>,
}

impl<R> PlutusMintingPolicy<R> {
    /// Constructor for new V1 [`PlutusMintingPolicy`] from a [`PlutusScriptFile`]
    pub fn new_v1(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v1_policy = PlutusMintingPolicy {
            version: TransactionVersion::V1,
            cbor: outer.to_vec(),
            _redeemer: Default::default(),
        };
        Ok(v1_policy)
    }

    /// Constructor for new V2 [`PlutusMintingPolicy`] from a [`PlutusScriptFile`]
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusMintingPolicy {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }

    /// Constructor for new V2 [`PlutusMintingPolicy`] from a [`ValidatorBlueprint`]
    pub fn from_blueprint(blueprint: ValidatorBlueprint) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(blueprint.compiled_code())
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusMintingPolicy {
            version: TransactionVersion::V2,
            cbor,
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }

    /// Constructor for new V2 [`PlutusMintingPolicy`] from a CBOR hex string
    pub fn v2_from_cbor(cbor: String) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(cbor).map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_policy = PlutusMintingPolicy {
            version: TransactionVersion::V2,
            cbor,
            _redeemer: Default::default(),
        };
        Ok(v2_policy)
    }
}

/// Builder for [`PlutusMintingPolicy`] that requires a single parameter
pub struct OneParamPlutusPolicy<One, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _one: PhantomData<One>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, R> OneParamPlutusPolicy<One, R> {
    /// Constructor for new V2 [`OneParamPlutusPolicy`] from a [`PlutusScriptFile`]
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamPlutusPolicy {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _one: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    /// Constructor for new V2 [`OneParamPlutusPolicy`] from a [`ValidatorBlueprint`]
    pub fn from_blueprint(blueprint: ValidatorBlueprint) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(blueprint.compiled_code())
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_val = OneParamPlutusPolicy {
            version: TransactionVersion::V2,
            cbor,
            _one: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_val)
    }

    /// Apply the singular parameter to the policy to get a complete [`PlutusMintingPolicy`]
    pub fn apply(&self, one: One) -> RawPlutusScriptResult<PlutusMintingPolicy<R>> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .unwrap()
                .into();
        let one_data: PlutusData = one.into();
        let one_term = Term::Constant(Rc::new(Constant::Data(one_data.into())));
        let program = program.apply_term(&one_term);
        let fake: Program<FakeNamedDeBruijn> = program.into();
        let new_cbor = fake
            .to_cbor()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let policy = PlutusMintingPolicy {
            version: self.version.clone(),
            cbor: new_cbor,
            _redeemer: Default::default(),
        };
        Ok(policy)
    }
}

/// Builder for [`PlutusMintingPolicy`] that requires two parameters
pub struct TwoParamMintingPolicy<One, Two, Redeemer> {
    version: TransactionVersion,
    cbor: Vec<u8>,
    _one: PhantomData<One>,
    _two: PhantomData<Two>,
    _redeemer: PhantomData<Redeemer>,
}

impl<One: Into<PlutusData>, Two: Into<PlutusData>, R> TwoParamMintingPolicy<One, Two, R> {
    /// Constructor for new V2 [`TwoParamMintingPolicy`] from a [`PlutusScriptFile`]
    pub fn new_v2(script_file: PlutusScriptFile) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(script_file.cborHex)
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let mut outer_decoder = Decoder::new(&cbor);
        let outer = outer_decoder
            .bytes()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_pol = TwoParamMintingPolicy {
            version: TransactionVersion::V2,
            cbor: outer.to_vec(),
            _one: Default::default(),
            _two: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_pol)
    }

    /// Constructor for new V2 [`TwoParamMintingPolicy`] from a [`ValidatorBlueprint`]
    pub fn from_blueprint(blueprint: ValidatorBlueprint) -> RawPlutusScriptResult<Self> {
        let cbor = hex::decode(blueprint.compiled_code())
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let v2_pol = TwoParamMintingPolicy {
            version: TransactionVersion::V2,
            cbor,
            _one: Default::default(),
            _two: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(v2_pol)
    }

    /// Apply a single parameter to the policy to get a [`OneParamPlutusPolicy`]
    pub fn apply(&self, one: One) -> RawPlutusScriptResult<OneParamPlutusPolicy<Two, R>> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .unwrap()
                .into();
        let one_data: PlutusData = one.into();
        let one_term = Term::Constant(Rc::new(Constant::Data(one_data.into())));
        let program = program.apply_term(&one_term);
        let fake: Program<FakeNamedDeBruijn> = program.into();
        let new_cbor = fake
            .to_cbor()
            .map_err(|e| PlutusScriptError::AikenApply(e.to_string()))?;
        let policy = OneParamPlutusPolicy {
            version: self.version.clone(),
            cbor: new_cbor,
            _one: Default::default(),
            _redeemer: Default::default(),
        };
        Ok(policy)
    }
}

impl<Redeemer> MintingPolicy<Redeemer> for PlutusMintingPolicy<Redeemer>
where
    Redeemer: Into<PlutusData> + Send + Sync,
{
    fn execute(&self, redeemer: Redeemer, ctx: TxContext) -> ScriptResult<ExecutionCost> {
        let program: Program<NamedDeBruijn> =
            Program::<FakeNamedDeBruijn>::from_cbor(&self.cbor, &mut Vec::new())
                .map_err(as_failed_to_execute)?
                .into();
        let redeemer_data: PlutusData = redeemer.into();
        let redeemer_term = Term::Constant(Rc::new(Constant::Data(redeemer_data.into())));
        let program = program.apply_term(&redeemer_term);
        let ctx_data: PlutusData = ctx.into();
        let ctx_term = Term::Constant(Rc::new(Constant::Data(ctx_data.into())));
        let program = program.apply_term(&ctx_term);
        let mut eval_result = match self.version {
            TransactionVersion::V1 => program.eval_version(&Language::PlutusV1),
            TransactionVersion::V2 => program.eval(ExBudget::default()), // TODO: parameterize
        };
        let logs = eval_result.logs();
        let cost = eval_result.cost();
        eval_result
            .result()
            .map_err(|e| PlutusScriptError::AikenEval {
                error: format!("{e:?}"),
                logs,
            })
            .map_err(as_failed_to_execute)?;
        Ok(cost.into())
    }

    fn id(&self) -> ScriptResult<String> {
        let cbor = self.script_hex().unwrap();
        let script = match self.version {
            TransactionVersion::V1 => {
                let script_bytes =
                    hex::decode(&cbor).map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v1 = PlutusV1Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v1(&v1)
            }
            TransactionVersion::V2 => {
                let script_bytes =
                    hex::decode(&cbor).map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                let v2 = PlutusV2Script::from_bytes(script_bytes)
                    .map_err(|e| ScriptError::IdRetrieval(e.to_string()))?;
                PlutusScript::from_v2(&v2)
            }
        };
        Ok(script.hash().to_string())
    }

    fn script_hex(&self) -> ScriptResult<String> {
        let wrap = Encoder::new(Vec::new())
            .bytes(&self.cbor)
            .map_err(|e| ScriptError::ScriptHexRetrieval(e.to_string()))?
            .clone()
            .into_writer();

        let hex = hex::encode(wrap);
        Ok(hex)
    }
}
