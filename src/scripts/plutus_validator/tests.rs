use super::*;
use crate::scripts::context::{
    pub_key_hash_from_address_if_available,
    ContextBuilder,
};

mod game;
mod hello;

#[test]
fn execute_always_succeeds() {
    let script_file = PlutusScriptFile {
        r#type: "PlutusScriptV1".to_string(),
        description: "".to_string(),
        cborHex: "4e4d01000033222220051200120011".to_string(),
    };
    let script: PlutusValidator<(), ()> = PlutusValidator::new_v1(script_file).unwrap();

    let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
    let signer_pkh = pub_key_hash_from_address_if_available(&signer).unwrap();

    let ctx = ContextBuilder::new(signer_pkh).build_spend(&[], 0);

    script.execute((), (), ctx).unwrap();
}
