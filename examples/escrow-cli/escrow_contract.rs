use naumachia::{
    address::{Address, ADA},
    logic::SCLogic,
    logic::{SCLogicError, SCLogicResult},
    output::Output,
    transaction::UnBuiltTransaction,
    txorecord::TxORecord,
    validator::{TxContext, ValidatorCode},
    validator::{ValidatorCodeError, ValidatorCodeResult},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashMap;

pub struct EscrowValidatorScript;

impl ValidatorCode<EscrowDatum, ()> for EscrowValidatorScript {
    fn execute(
        &self,
        datum: EscrowDatum,
        _redeemer: (),
        ctx: TxContext,
    ) -> ValidatorCodeResult<()> {
        signer_is_recipient(&datum, &ctx)?;
        Ok(())
    }

    fn address(&self) -> Address {
        Address::new("escrow validator")
    }
}

fn signer_is_recipient(datum: &EscrowDatum, ctx: &TxContext) -> ValidatorCodeResult<()> {
    if datum.receiver != ctx.signer {
        Err(ValidatorCodeError::FailedToExecute(format!(
            "Signer: {:?} doesn't match receiver: {:?}",
            ctx.signer, datum.receiver
        )))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EscrowContract;

#[allow(dead_code)]
#[derive(Clone)]
pub enum EscrowEndpoint {
    Escrow { amount: u64, receiver: Address },
    Claim { output_id: String },
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct EscrowDatum {
    receiver: Address,
}

impl EscrowDatum {
    pub fn receiver(&self) -> &Address {
        &self.receiver
    }
}

#[derive(Debug, Error)]
enum EscrowContractError {
    #[error("Output with ID {0:?} not found.")]
    OutputNotFound(String),
}

impl SCLogic for EscrowContract {
    type Endpoint = EscrowEndpoint;
    type Lookup = ();
    type LookupResponse = Vec<Output<Self::Datum>>;
    type Datum = EscrowDatum;
    type Redeemer = ();

    fn handle_endpoint<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        txo_record: &Record,
    ) -> SCLogicResult<UnBuiltTransaction<EscrowDatum, ()>> {
        match endpoint {
            EscrowEndpoint::Escrow { amount, receiver } => escrow(amount, receiver),
            EscrowEndpoint::Claim { output_id } => claim(&output_id, txo_record),
        }
    }

    fn lookup<Record: TxORecord<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        txo_record: &Record,
    ) -> SCLogicResult<Self::LookupResponse> {
        let outputs = txo_record.outputs_at_address(&EscrowValidatorScript.address());
        Ok(outputs)
    }
}

fn escrow(amount: u64, receiver: Address) -> SCLogicResult<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = EscrowValidatorScript;
    let address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let datum = EscrowDatum { receiver };
    let mut values = HashMap::new();
    values.insert(ADA, amount);
    let u_tx = UnBuiltTransaction::default().with_script_init(datum, values, address);
    Ok(u_tx)
}

fn claim<Record: TxORecord<EscrowDatum, ()>>(
    output_id: &str,
    txo_record: &Record,
) -> SCLogicResult<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = Box::new(EscrowValidatorScript);
    let output = lookup_output(output_id, txo_record)?;
    let u_tx = UnBuiltTransaction::default().with_script_redeem(output, (), script);
    Ok(u_tx)
}

fn lookup_output<Record: TxORecord<EscrowDatum, ()>>(
    id: &str,
    txo_record: &Record,
) -> SCLogicResult<Output<EscrowDatum>> {
    let script_address = EscrowValidatorScript.address();
    let outputs = txo_record.outputs_at_address(&script_address);
    outputs
        .iter()
        .find(|o| o.id() == id)
        .cloned()
        .ok_or_else(|| {
            SCLogicError::Lookup(Box::new(EscrowContractError::OutputNotFound(
                id.to_string(),
            )))
        })
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use naumachia::backend::in_memory_record::TestBackendsBuilder;
    use naumachia::smart_contract::{SmartContract, SmartContractTrait};
    use naumachia::txorecord::TxORecord;

    #[test]
    fn escrow__can_create_instance() {
        let me = Address::new("me");
        let alice = Address::new("alice");
        let start_amount = 100;
        let mut backend = TestBackendsBuilder::new(&me)
            .start_output(&me)
            .with_value(ADA, start_amount)
            .finish_output()
            .build();

        let escrow_amount = 25;
        let call = EscrowEndpoint::Escrow {
            amount: escrow_amount,
            receiver: alice.clone(),
        };
        let script = EscrowValidatorScript;
        let contract = SmartContract::new(&EscrowContract, &backend);
        contract.hit_endpoint(call).unwrap();

        let escrow_address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
        let expected = escrow_amount;
        let actual = backend
            .txo_record
            .balance_at_address(&script.address(), &ADA);
        assert_eq!(expected, actual);

        let expected = start_amount - escrow_amount;
        let actual = backend.txo_record.balance_at_address(&me, &ADA);
        assert_eq!(expected, actual);

        let instance = backend
            .txo_record
            .outputs_at_address(&script.address())
            .pop()
            .unwrap();
        // The creator tries to spend escrow but fails because not recipient
        let call = EscrowEndpoint::Claim {
            output_id: instance.id().to_string(),
        };

        let contract = SmartContract::new(&EscrowContract, &backend);
        let attempt = contract.hit_endpoint(call.clone());
        assert!(attempt.is_err());

        // The recipient tries to spend and succeeds
        backend.txo_record.signer = alice.clone();
        let contract = SmartContract::new(&EscrowContract, &backend);
        contract.hit_endpoint(call).unwrap();

        let alice_balance = backend.txo_record.balance_at_address(&alice, &ADA);
        assert_eq!(alice_balance, escrow_amount);

        let script_balance = backend.txo_record.balance_at_address(&escrow_address, &ADA);
        assert_eq!(script_balance, 0);
    }
}
