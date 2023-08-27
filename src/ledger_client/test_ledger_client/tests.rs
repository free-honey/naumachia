#![allow(non_snake_case)]

use super::*;
use crate::scripts::{ExecutionCost, MintingPolicy, ScriptError, ScriptResult, ValidatorCode};
use crate::transaction::TransactionVersion;
use crate::{
    ledger_client::{
        test_ledger_client::{local_persisted_storage::starting_output, TestLedgerClient},
        LedgerClient,
    },
    output::UnbuiltOutput,
    PolicyId, UnbuiltTransaction,
};

const ALICE: &str = "addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr";
const BOB: &str = "addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua";

const BLOCK_LENGTH: i64 = 1000;

#[tokio::test]
async fn outputs_at_address() {
    let signer = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let output = starting_output::<()>(&signer, starting_amount);
    let outputs = vec![(signer.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(signer.clone(), outputs, BLOCK_LENGTH, 0);
    let mut outputs = record.all_outputs_at_address(&signer).await.unwrap();
    assert_eq!(outputs.len(), 1);
    let first_output = outputs.pop().unwrap();
    let expected = starting_amount;
    let actual = first_output.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn balance_at_address() {
    let signer = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let output = starting_output::<()>(&signer, starting_amount);
    let outputs = vec![(signer.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(signer.clone(), outputs, BLOCK_LENGTH, 0);
    let expected = starting_amount;
    let actual = record
        .balance_at_address(&signer, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn issue_transfer() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;
    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, transfer_amount);
    let recipient = Address::from_bech32(BOB).unwrap();
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
    let actual_bob_ada = actual_bob.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(actual_bob_ada, transfer_amount);
    let actual_bob_tx_hash = actual_bob.id().tx_hash();

    let actual_alice = record
        .all_outputs_at_address(&sender)
        .await
        .unwrap()
        .pop()
        .unwrap();
    let actual_alice_ada = actual_alice.values().get(&PolicyId::Lovelace).unwrap();
    assert_eq!(actual_alice_ada, starting_amount - transfer_amount);

    let actual_alice_tx_hash = actual_alice.id().tx_hash();

    assert_eq!(actual_bob_tx_hash, actual_alice_tx_hash);
}

#[tokio::test]
async fn issuing_tx_advances_time_by_block_length() {
    let signer = Address::from_bech32(ALICE).unwrap();
    let outputs = vec![];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(signer.clone(), outputs, BLOCK_LENGTH, 0);
    let starting_time = record.current_time().await.unwrap();
    let tx = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![],
        minting: Default::default(),
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();
    let expected = starting_time + BLOCK_LENGTH;
    let actual = record.current_time().await.unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn errors_if_spending_more_than_you_own() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let transfer_amount = 3_000_000;
    let outputs = vec![];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, transfer_amount);
    let recipient = Address::from_bech32(BOB).unwrap();
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
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let current_time = 5;
    let valid_time = 10;
    record.set_current_time(current_time).await.unwrap();

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, transfer_amount);
    let recipient = Address::from_bech32(BOB).unwrap();
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
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let transfer_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let current_time = 10;
    let valid_time = 5;
    record.set_current_time(current_time).await.unwrap();

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, transfer_amount);
    let recipient = Address::from_bech32(BOB).unwrap();
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
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<ExecutionCost> {
        Ok(ExecutionCost::default())
    }

    fn address(&self, _network: Network) -> ScriptResult<Address> {
        Ok(
            Address::from_bech32("addr_test1wrme5jjggy97th309h2dwpv57wsphxskuc8jkw00c2kn47gu8mkzu")
                .unwrap(),
        )
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn redeeming_datum() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, locking_amount);

    let validator = AlwaysTrueFakeValidator;
    let network = Network::Testnet;

    let script_address = validator.address(network).unwrap();
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
        .balance_at_address(&sender, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::Lovelace)
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
        .balance_at_address(&sender, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(script_balance, 0);
}

struct AlwaysFailsFakeValidator;

impl ValidatorCode<(), ()> for AlwaysFailsFakeValidator {
    fn execute(&self, _datum: (), _redeemer: (), _ctx: TxContext) -> ScriptResult<ExecutionCost> {
        Err(ScriptError::FailedToExecute(
            "Should always fail!".to_string(),
        ))
    }

    fn address(&self, _network: Network) -> ScriptResult<Address> {
        Ok(
            Address::from_bech32("addr_test1wrme5jjggy97th309h2dwpv57wsphxskuc8jkw00c2kn47gu8mkzu")
                .unwrap(),
        )
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn failing_script_will_not_redeem() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, locking_amount);

    let validator = AlwaysFailsFakeValidator;

    let network = Network::Testnet;
    let script_address = validator.address(network).unwrap();
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
        .balance_at_address(&sender, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::Lovelace)
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
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let locking_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, locking_amount);

    let validator = AlwaysTrueFakeValidator;

    let network = Network::Testnet;
    let script_address = validator.address(network).unwrap();
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
        .balance_at_address(&sender, &PolicyId::Lovelace)
        .await
        .unwrap();
    assert_eq!(alice_balance, starting_amount - locking_amount);
    let script_balance = record
        .balance_at_address(&script_address, &PolicyId::Lovelace)
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

pub struct AlwaysTruePolicy;

impl MintingPolicy<()> for AlwaysTruePolicy {
    fn execute(&self, _redeemer: (), _ctx: TxContext) -> ScriptResult<ExecutionCost> {
        Ok(ExecutionCost::default())
    }

    fn id(&self) -> ScriptResult<String> {
        Ok(hex::encode(vec![1, 1, 1, 1, 1]))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn mint_always_true() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let minting_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let policy = AlwaysTruePolicy;
    let id = policy.id().unwrap();

    let script_box: Box<dyn MintingPolicy<()>> = Box::new(policy);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![],
        minting: vec![(minting_amount, None, (), script_box)],
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::NativeToken(id, None))
        .await
        .unwrap();
    assert_eq!(alice_balance, minting_amount);
}

pub struct AlwaysFailsPolicy;

impl MintingPolicy<()> for AlwaysFailsPolicy {
    fn execute(&self, _redeemer: (), _ctx: TxContext) -> ScriptResult<ExecutionCost> {
        Err(ScriptError::FailedToExecute("Always fails :@".to_string()))
    }

    fn id(&self) -> ScriptResult<String> {
        Ok(hex::encode(vec![2, 2, 2, 2, 2]))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn mint_always_fails_errors() {
    let sender = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let minting_amount = 3_000_000;

    let output = starting_output::<()>(&sender, starting_amount);
    let outputs = vec![(sender.clone(), output)];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(sender.clone(), outputs, BLOCK_LENGTH, 0);

    let policy = AlwaysFailsPolicy;
    let id = policy.id().unwrap();

    let script_box: Box<dyn MintingPolicy<()>> = Box::new(policy);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![],
        unbuilt_outputs: vec![],
        minting: vec![(minting_amount, None, (), script_box)],
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap_err();

    let alice_balance = record
        .balance_at_address(&sender, &PolicyId::NativeToken(id, None))
        .await
        .unwrap();
    assert_eq!(alice_balance, 0);
}

pub struct SpendsNFTPolicy {
    policy_id: String,
}

impl MintingPolicy<()> for SpendsNFTPolicy {
    fn execute(&self, _redeemer: (), ctx: TxContext) -> ScriptResult<ExecutionCost> {
        if ctx
            .inputs
            .iter()
            .any(|input| input.value.inner.contains_key(&self.policy_id))
        {
            Ok(ExecutionCost::default())
        } else {
            Err(ScriptError::FailedToExecute("input not found".to_string()))
        }
    }

    fn id(&self) -> ScriptResult<String> {
        Ok(hex::encode(vec![3, 3, 3, 3, 3]))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn spends_specific_script_value() {
    let minter = Address::from_bech32(ALICE).unwrap();
    let starting_amount = 10_000_000;
    let minting_amount = 3_000_000;

    let nft_policy_id = "my_nft".to_string();
    let validator = AlwaysTrueFakeValidator;
    let network = Network::Testnet;
    let val_address = validator.address(network).unwrap();
    let mut values = Values::default();
    let policy = PolicyId::NativeToken(nft_policy_id.clone(), None);
    values.add_one_value(&policy, 1);
    let input = Output::new_validator(vec![1, 2, 3, 4], 0, val_address.clone(), values, ());

    let output = starting_output::<()>(&minter, starting_amount);
    let outputs = vec![(minter.clone(), output), (val_address, input.clone())];
    let record: TestLedgerClient<(), (), _> =
        TestLedgerClient::new_in_memory(minter.clone(), outputs, BLOCK_LENGTH, 0);

    let policy = SpendsNFTPolicy {
        policy_id: nft_policy_id,
    };
    let id = policy.id().unwrap();
    let asset_name = None;

    let script_box: Box<dyn MintingPolicy<()>> = Box::new(policy);

    let boxed_validator: Box<dyn ValidatorCode<(), ()>> = Box::new(validator);
    let redeem_info = (input, (), boxed_validator);
    let tx: UnbuiltTransaction<(), ()> = UnbuiltTransaction {
        script_version: TransactionVersion::V2,
        script_inputs: vec![redeem_info],
        unbuilt_outputs: vec![],
        minting: vec![(minting_amount, asset_name.clone(), (), script_box)],
        specific_wallet_inputs: vec![],
        valid_range: (None, None),
    };
    record.issue(tx).await.unwrap();

    let alice_balance = record
        .balance_at_address(&minter, &PolicyId::NativeToken(id, asset_name))
        .await
        .unwrap();
    assert_eq!(alice_balance, minting_amount);
}
