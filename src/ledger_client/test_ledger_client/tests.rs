#![allow(non_snake_case)]

use super::*;
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

    assert!(matches!(
        LedgerClientError::FailedToIssueTx(Box::new(TestLCError::NotEnoughInputs)),
        error
    ));
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
        script_version: TransactionVersion::V1,
        script_inputs: vec![],
        unbuilt_outputs: vec![new_output],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, Some(valid_time)),
    };
    let error = record.issue(tx).await.unwrap_err();

    assert!(matches!(error, LedgerClientError::FailedToIssueTx(_),));
}
