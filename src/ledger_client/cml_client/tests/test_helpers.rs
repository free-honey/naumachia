use crate::ledger_client::cml_client::validator_script::{PlutusScriptFile, RawPlutusValidator};
use crate::{
    ledger_client::cml_client::key_manager::TESTNET,
    output::{Output, UnbuiltOutput},
    scripts::ValidatorCode,
    values::Values,
    Address, PolicyId, UnbuiltTransaction,
};
use cardano_multiplatform_lib::{
    address::Address as CMLAddress,
    address::{EnterpriseAddress, StakeCredential},
    plutus::{PlutusScript, PlutusV1Script},
};
use std::{fs::File, io::Read};

pub fn transfer_tx(recipient: Address, amount: u64) -> UnbuiltTransaction<(), ()> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let output = UnbuiltOutput::new_wallet(recipient, values);
    UnbuiltTransaction {
        script_inputs: vec![],
        unbuilt_outputs: vec![output],
        minting: Default::default(),
        policies: Default::default(),
    }
}

pub fn lock_at_always_succeeds_tx(amount: u64) -> UnbuiltTransaction<(), ()> {
    let script_address = always_succeeds_script_address(TESTNET);
    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, amount);
    let output = UnbuiltOutput::new_validator(script_address, values, ());
    UnbuiltTransaction {
        script_inputs: vec![],
        unbuilt_outputs: vec![output],
        minting: Default::default(),
        policies: Default::default(),
    }
}

pub fn always_succeeds_script_cml_address(network: u8) -> CMLAddress {
    let script = always_succeeds_script();
    let script_hash = script.hash();
    let stake_cred = StakeCredential::from_scripthash(&script_hash);
    let enterprise_addr = EnterpriseAddress::new(network, &stake_cred);
    enterprise_addr.to_address()
}

pub fn always_succeeds_script_address(network: u8) -> Address {
    let cml_script_address = always_succeeds_script_cml_address(network);
    let script_address_str = cml_script_address.to_bech32(None).unwrap();
    Address::Script(script_address_str)
}

pub fn always_succeeds_hex() -> String {
    let script_file = read_script_from_file("./plutus/always-succeeds-spending.plutus");
    script_file.cborHex
}

pub fn always_succeeds_script() -> PlutusScript {
    let script_file = read_script_from_file("./plutus/always-succeeds-spending.plutus");
    let script_hex = script_file.cborHex;
    let script_bytes = hex::decode(&script_hex).unwrap();
    let v1 = PlutusV1Script::from_bytes(script_bytes).unwrap();
    PlutusScript::from_v1(&v1)
}

pub fn read_script_from_file(file_path: &str) -> PlutusScriptFile {
    let mut file = File::open(file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    serde_json::from_str(&data).unwrap()
}

pub fn claim_always_succeeds_datum_tx(script_input: &Output<()>) -> UnbuiltTransaction<(), ()> {
    let script = RawPlutusValidator::new_v1(always_succeeds_hex()).unwrap();
    let script = Box::new(script) as Box<dyn ValidatorCode<(), ()>>;
    UnbuiltTransaction {
        script_inputs: vec![(script_input.clone(), (), script)],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        policies: Default::default(),
    }
}

pub fn output_from_tx<'a, D>(tx_id: &'a str, outputs: &'a Vec<Output<D>>) -> Option<&'a Output<D>> {
    for output in outputs {
        let id = output.id();
        let tx_hash = id.tx_hash();
        if tx_hash == tx_id {
            return Some(output);
        }
    }
    None
}
