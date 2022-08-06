use naumachia::{
    address::{Address, ADA},
    error::Result as NauResult,
    logic::SCLogic,
    output::Output,
    transaction::UnBuiltTransaction,
    validator::{TxContext, ValidatorCode},
};
use std::collections::HashMap;

pub struct EscrowValidatorScript;

impl ValidatorCode<EscrowDatum, ()> for EscrowValidatorScript {
    fn execute(&self, datum: EscrowDatum, _redeemer: (), ctx: TxContext) -> NauResult<()> {
        signer_is_recipient(&datum, &ctx)?;
        Ok(())
    }

    fn address(&self) -> Address {
        Address::new("escrow validator")
    }
}

fn signer_is_recipient(datum: &EscrowDatum, ctx: &TxContext) -> NauResult<()> {
    if datum.receiver != ctx.signer {
        Err(format!(
            "Signer: {:?} doesn't match receiver: {:?}",
            ctx.signer, datum.receiver
        ))
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct EscrowContract;

#[derive(Clone)]
pub enum Endpoint {
    Escrow { amount: u64, receiver: Address },
    Claim { output: Output<EscrowDatum> },
}

#[derive(Clone, PartialEq, Debug)]
pub struct EscrowDatum {
    receiver: Address,
}

impl SCLogic for EscrowContract {
    type Endpoint = Endpoint;
    type Datum = EscrowDatum;
    type Redeemer = ();

    fn handle_endpoint(
        endpoint: Self::Endpoint,
        _issuer: &Address,
    ) -> NauResult<UnBuiltTransaction<EscrowDatum, ()>> {
        match endpoint {
            Endpoint::Escrow { amount, receiver } => escrow(amount, receiver),
            Endpoint::Claim { output } => claim(output),
        }
    }
}

fn escrow(amount: u64, receiver: Address) -> NauResult<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = EscrowValidatorScript;
    let address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let datum = EscrowDatum { receiver };
    let mut values = HashMap::new();
    values.insert(ADA, amount);
    let u_tx = UnBuiltTransaction::default().with_script_init(datum, values, address);
    Ok(u_tx)
}

fn claim(output: Output<EscrowDatum>) -> NauResult<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = Box::new(EscrowValidatorScript);
    let u_tx = UnBuiltTransaction::default().with_script_redeem(output, (), script);
    Ok(u_tx)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use naumachia::backend::fake_backend::TestBackendsBuilder;
    use naumachia::backend::TxORecord;
    use naumachia::smart_contract::{SmartContract, SmartContractTrait};

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
        let call = Endpoint::Escrow {
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
        let call = Endpoint::Claim { output: instance };

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
