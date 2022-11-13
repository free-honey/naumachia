use super::*;

mod game;
mod hello;

#[test]
fn execute_always_succeeds() {
    let script_file = PlutusScriptFile {
        r#type: "PlutusScriptV1".to_string(),
        description: "".to_string(),
        cborHex: "4e4d01000033222220051200120011".to_string(),
    };
    let script = RawPlutusValidator::new_v1(script_file).unwrap();

    let ctx = TxContext {
        signer: Address::Raw("placeholder".to_string()),
    };

    script.execute((), (), ctx).unwrap();
}
