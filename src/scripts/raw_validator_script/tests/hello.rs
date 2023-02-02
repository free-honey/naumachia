use crate::scripts::context::ContextBuilder;
use crate::scripts::raw_script::PlutusScriptFile;
use crate::scripts::raw_validator_script::RawPlutusValidator;
use crate::scripts::{ScriptError, ValidatorCode};
use pallas_addresses::Address;

/// run :: HelloDatum -> HelloRedeemer -> ScriptContext -> Bool
/// run (HelloDatum datum) (HelloRedeemer redeemer) _ = redeemer < datum
fn hello_script_file() -> PlutusScriptFile {
    PlutusScriptFile {
        r#type: "PlutusScriptV1".to_string(),
        description: "".to_string(),
        cborHex: "588b5889010000323232232232232533500110081326320083357389210350543500008323233357\
        3466e20008004488008488004cccd5cd19b8735573aa00a900011bad357426aae7940188c98c8024cd5ce00500\
        48039999ab9a3370e6aae75400920002375a6ae84d55cf280191931900419ab9c0090080061375400226ea8005\
        2612001491035054310001"
            .to_string(),
    }
}

// TODO: The Redeemer and Datum need to be wrapped in Constrs
//  https://github.com/MitchTurner/naumachia/issues/80
#[ignore]
#[test]
fn execute_hello_passes() {
    let script_file = hello_script_file();
    let script = RawPlutusValidator::new_v1(script_file).unwrap();

    let datum: u64 = 50;
    let redeemer: u64 = 49;
    let signer = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();
    let ctx = ContextBuilder::new(signer).build_spend(&vec![], 0);
    script.execute(datum, redeemer, ctx).unwrap();
}

// TODO: The Redeemer and Datum need to be wrapped in Constrs
//  https://github.com/MitchTurner/naumachia/issues/80
#[ignore]
#[test]
fn execute_hello_fails() {
    let script_file = hello_script_file();
    let script = RawPlutusValidator::new_v1(script_file).unwrap();

    let datum: u64 = 50;
    let redeemer: u64 = 51;

    let signer = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();
    let ctx = ContextBuilder::new(signer).build_spend(&vec![], 0);

    // PT5: 'check' input is 'False'
    assert_eq!(
        script.execute(datum, redeemer, ctx).unwrap_err(),
        ScriptError::FailedToExecute(
            "AikenEval { error: \"EvaluationFailure\", logs: [\"PT5\"] }".to_string()
        )
    );
}
