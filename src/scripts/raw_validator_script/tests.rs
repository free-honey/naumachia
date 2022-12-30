use super::*;
use crate::scripts::ContextBuilder;

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

    let signer = Address::new("placeholder");

    let ctx = ContextBuilder::new(signer).build();

    script.execute((), (), ctx).unwrap();
}
