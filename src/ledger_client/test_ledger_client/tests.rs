#![allow(non_snake_case)]

use super::*;
use crate::scripts::{ScriptError, ScriptResult, ValidatorCode};
use crate::transaction::TransactionVersion;
use crate::{
    ledger_client::{
        test_ledger_client::{local_persisted_storage::starting_output, TestLedgerClient},
        LedgerClient,
    },
    output::UnbuiltOutput,
    PolicyId, UnbuiltTransaction,
};

#[tokio::test]
async fn outputs_at_address() {
    let signer = Address::new("alice");
    let starting_amount = 10_000_000;
    let output = starting_output::<()>(&signer, starting_amount);
    let outputs = vec![(signer.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(signer.clone(), outputs);
    let mut outputs = record.all_outputs_at_address(&signer).await.unwrap();
    assert_eq!(outputs.len(), 1);
    let first_output = outputs.pop().unwrap();
    let expected = starting_amount;
    let actual = first_output.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn balance_at_address() {
    let signer = Address::new("alice");
    let starting_amount = 10_000_000;
    let output = starting_output::<()>(&signer, starting_amount);
    let outputs = vec![(signer.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(signer.clone(), outputs);
    let expected = starting_amount;
    let actual = record
        .balance_at_address(&signer, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn issue_transfer() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;
    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, transfer_amount);
    let recipient = Address::new("bob");
    let new_output = UnbuiltOutput::new_wallet(recipient.clone(), values);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();
    let actual_bob = record
        .all_outputs_at_address(&recipient)
        .await
        .unwrap()
        .pop()
        .unwrap();
    let actual_bob_ada = actual_bob.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(actual_bob_ada, transfer_amount);
    let actual_bob_tx_hash = actual_bob.id().tx_hash();

    let actual_alice = record
        .all_outputs_at_address(&sender)
        .await
        .unwrap()
        .pop()
        .unwrap();
    let actual_alice_ada = actual_alice.values().get(&PolicyId::ADA).unwrap();
    assert_eq!(actual_alice_ada, starting_amount - transfer_amount);

    let actual_alice_tx_hash = actual_alice.id().tx_hash();

    assert_eq!(actual_bob_tx_hash, actual_alice_tx_hash);
}

#[tokio::test]
async fn errors_if_spending_more_than_you_own() {
    let sender = Address::new("alice");
    let transfer_amount = 3_000_000;
    let outputs = vec![];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, transfer_amount);
    let recipient = Address::new("bob");
    let new_output = UnbuiltOutput::new_wallet(recipient.clone(), values);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    let error = record.issue(tx).await.unwrap_err();

    assert!(matches!(error, LedgerClientError::FailedToIssueTx(_),));
}

#[tokio::test]
async fn cannot_transfer_before_valid_range() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let mut record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let current_time = 5;
    let valid_time = 10;
    record.set_current_time(current_time).await.unwrap();

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, transfer_amount);
    let recipient = Address::new("bob");
    let new_output = UnbuiltOutput::new_wallet(recipient.clone(), values);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V1,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (Some(valid_time), None),
    };
    let error = record.issue(tx).await.unwrap_err();

    assert!(matches!(error, LedgerClientError::FailedToIssueTx(_)));
}

#[tokio::test]
async fn cannot_transfer_after_valid_range() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let mut record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let current_time = 10;
    let valid_time = 5;
    record.set_current_time(current_time).await.unwrap();

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, transfer_amount);
    let recipient = Address::new("bob");
    let new_output = UnbuiltOutput::new_wallet(recipient.clone(), values);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, Some(valid_time)),
    };
    let error = record.issue(tx).await.unwrap_err();

    assert!(matches!(error, LedgerClientError::FailedToIssueTx(_),));
}

#[derive(Clone, Copy)]
struct AlwaysTrueFakeValidator;

impl ValidatorCode<(), ()> for AlwaysTrueFakeValidator {
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        Ok(Address::new("script"))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn redeeming_datum() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, locking_amount);

    let validator = AlwaysTrueFakeValidator;

    let script_address = validator.address(0).unwrap();
    let new_output = UnbuiltOutput::new_validator(script_address.clone(), values, ());
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V1,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(script_balance, locking_amount);

    let output = record
        .all_outputs_at_address(&script_address)
        .await
        .unwrap()
        .pop()
        .unwrap();

    let script_box: Box<dyn ValidatorCode<(), ()>> = Box::new(validator);
    let redemption_details = (output, (), script_box);

    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![redemption_details],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };

    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(script_balance, 0);
}

struct AlwaysFailsFakeValidator;

impl ValidatorCode<(), ()> for AlwaysFailsFakeValidator {
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<()> {
        Err(ScriptError::FailedToExecute(
            "Should always fail!".to_string(),
        ))
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        Ok(Address::new("script"))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn failing_script_will_not_redeem() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, locking_amount);

    let validator = AlwaysFailsFakeValidator;

    let script_address = validator.address(0).unwrap();
    let new_output = UnbuiltOutput::new_validator(script_address.clone(), values, ());
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(script_balance, locking_amount);

    let output = record
        .all_outputs_at_address(&script_address)
        .await
        .unwrap()
        .pop()
        .unwrap();

    let script_box: Box<dyn ValidatorCode<(), ()>> = Box::new(validator);
    let redemption_details = (output, (), script_box);

    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![redemption_details],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };

    record.issue(tx).await.unwrap_err();
}

#[tokio::test]
async fn cannot_redeem_datum_twice() {
    let sender = Address::new("alice");
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::ADA, locking_amount);

    let validator = AlwaysTrueFakeValidator;

    let script_address = validator.address(0).unwrap();
    let new_output = UnbuiltOutput::new_validator(script_address.clone(), values, ());
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(script_balance, locking_amount);

    let output = record
        .all_outputs_at_address(&script_address)
        .await
        .unwrap()
        .pop()
        .unwrap();

    // When
    let script_box_1: Box<dyn ValidatorCode<(), ()>> = Box::new(validator);
    let script_box_2: Box<dyn ValidatorCode<(), ()>> = Box::new(validator);
    let redemption_details_1 = (output.clone(), (), script_box_1);
    let redemption_details_2 = (output, (), script_box_2);

    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V1,
        script_inputs: vec![redemption_details_1, redemption_details_2],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };

    // Then should error
    record.issue(tx).await.unwrap_err();
}
