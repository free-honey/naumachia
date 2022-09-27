use cardano_multiplatform_lib::address::{EnterpriseAddress, StakeCredential};
use cardano_multiplatform_lib::plutus::{PlutusScript, PlutusV1Script};
use naumachia::address::Address;
use naumachia::ledger_client::cml_client::validator_script::PlutusScriptFile;
use naumachia::scripts::{ScriptError, ScriptResult, TxContext, ValidatorCode};

const SCRIPT_RAW: &str = include_str!("../../plutus/always-succeeds-spending.plutus");

// TODO: This whole type won't be necessary once Aiken eval is added to Naumachia
pub struct AlwaysSucceedsScript {
    script_hex: String,
    cml_script: PlutusScript,
}

impl AlwaysSucceedsScript {
    pub fn new() -> ScriptResult<Self> {
        let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
            .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
        let script_hex = script_file.cborHex;
        let script_bytes =
            hex::decode(&script_hex).map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
        let v1 = PlutusV1Script::from_bytes(script_bytes)
            .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
        let cml_script = PlutusScript::from_v1(&v1);
        let v1_val = AlwaysSucceedsScript {
            script_hex,
            cml_script,
        };
        Ok(v1_val)
    }
}

impl ValidatorCode<(), ()> for AlwaysSucceedsScript {
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, network: u8) -> ScriptResult<Address> {
        let script_hash = self.cml_script.hash();
        let stake_cred = StakeCredential::from_scripthash(&script_hash);
        let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
        let cml_script_address = enterprise_addr.to_address();
        let script_address_str = cml_script_address.to_bech32(None).unwrap();
        let address = Address::Script(script_address_str);
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<&str> {
        Ok(&self.script_hex)
    }
}
