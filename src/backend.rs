use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

use crate::{
    address::{Address, Policy},
    error::Error,
    error::Result,
    output::Output,
    transaction::Action,
    txorecord::{TxORecord, TxORecordError},
    validator::{TxContext, ValidatorCode},
    Transaction, UnBuiltTransaction,
};

pub mod in_memory_record;
pub mod local_persisted_record;

#[cfg(test)]
mod tests;

// TODO: These should all be fallible

#[derive(Debug)]
pub struct Backend<Datum, Redeemer: Clone + Eq, Record: TxORecord<Datum, Redeemer>> {
    // TODO: Make fields private
    pub _datum: PhantomData<Datum>,
    pub _redeemer: PhantomData<Redeemer>,
    pub txo_record: Record,
}

impl<Datum, Redeemer, Record> Backend<Datum, Redeemer, Record>
where
    Datum: Clone,
    Redeemer: Clone + Eq,
    Record: TxORecord<Datum, Redeemer>,
{
    pub fn new(txo_record: Record) -> Self {
        Backend {
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            txo_record,
        }
    }

    pub fn process(&self, u_tx: UnBuiltTransaction<Datum, Redeemer>) -> Result<()> {
        let tx = self.build(u_tx)?;
        self.txo_record.issue(tx).map_err(|e| Error::TxORecord(e))?;
        Ok(())
    }

    pub fn txo_record(&self) -> &Record {
        &self.txo_record
    }

    pub fn signer(&self) -> &Address {
        self.txo_record.signer()
    }

    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn handle_actions(
        &self,
        actions: Vec<Action<Datum, Redeemer>>,
    ) -> Result<(
        Vec<Output<Datum>>,
        Vec<Output<Datum>>,
        Vec<(Output<Datum>, Redeemer)>,
        HashMap<Address, Box<dyn ValidatorCode<Datum, Redeemer>>>,
    )> {
        let mut min_input_values: HashMap<Policy, u64> = HashMap::new();
        let mut min_output_values: HashMap<Address, RefCell<HashMap<Policy, u64>>> = HashMap::new();
        let mut script_inputs: Vec<Output<Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Datum>> = Vec::new();

        let mut redeemers = Vec::new();
        let mut scripts = HashMap::new();
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy,
                } => {
                    // Input
                    add_to_map(&mut min_input_values, policy.clone(), amount);

                    // Output
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::Mint {
                    amount,
                    recipient,
                    policy,
                } => {
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    for (policy, amount) in values.iter() {
                        add_to_map(&mut min_input_values, policy.clone(), *amount);
                    }
                    let id = Uuid::new_v4().to_string(); // TODO: This should be done by the TxORecord impl or something
                    let owner = address;
                    let output = Output::Validator {
                        id,
                        owner,
                        values,
                        datum,
                    };
                    specific_outputs.push(output);
                }
                Action::RedeemScriptOutput {
                    output,
                    redeemer,
                    script,
                } => {
                    script_inputs.push(output.clone());
                    let script_address = script.address();
                    redeemers.push((output, redeemer));
                    scripts.insert(script_address, script);
                }
            }
        }
        // inputs
        let (inputs, remainders) =
            self.select_inputs_for_one(self.txo_record.signer(), &min_input_values, script_inputs)?;

        // outputs
        remainders.iter().for_each(|(amt, recp, policy)| {
            add_amount_to_nested_map(&mut min_output_values, *amt, recp, policy)
        });

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = self.create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        Ok((inputs, outputs, redeemers, scripts))
    }

    // LOL Super Naive Solution, just select ALL inputs!
    // TODO: Use Random Improve prolly: https://cips.cardano.org/cips/cip2/
    //       but this is _good_enough_ for tests.
    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn select_inputs_for_one(
        &self,
        address: &Address,
        values: &HashMap<Policy, u64>,
        script_inputs: Vec<Output<Datum>>,
    ) -> Result<(Vec<Output<Datum>>, Vec<(u64, Address, Policy)>)> {
        let mut address_values = HashMap::new();
        let mut all_available_outputs = self.txo_record.outputs_at_address(address);
        all_available_outputs.extend(script_inputs);
        all_available_outputs
            .clone()
            .into_iter()
            .flat_map(|o| o.values().clone().into_iter().collect::<Vec<_>>())
            .for_each(|(policy, amount)| {
                add_to_map(&mut address_values, policy, amount);
            });
        let mut remainders = Vec::new();

        // TODO: REfactor :(
        for (policy, amt) in values.iter() {
            if let Some(available) = address_values.remove(policy) {
                if amt <= &available {
                    let remaining = available - amt;
                    remainders.push((remaining, address.clone(), policy.clone()));
                } else {
                    return Err(Error::InsufficientAmountOf(policy.to_owned()));
                }
            } else {
                return Err(Error::InsufficientAmountOf(policy.to_owned()));
            }
        }
        let other_remainders: Vec<_> = address_values
            .into_iter()
            .map(|(policy, amt)| (amt, address.clone(), policy))
            .collect();
        remainders.extend(other_remainders);
        Ok((all_available_outputs, remainders))
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(Policy, u64)>)>,
    ) -> Result<Vec<Output<Datum>>> {
        let outputs = values
            .into_iter()
            .map(|(owner, val_vec)| {
                let values = val_vec.into_iter().collect();
                let id = Uuid::new_v4().to_string(); // TODO: This should be done by the TxORecord impl or something
                Output::new_wallet(id, owner, values)
            })
            .collect();
        Ok(outputs)
    }

    fn build(
        &self,
        unbuilt_tx: UnBuiltTransaction<Datum, Redeemer>,
    ) -> Result<Transaction<Datum, Redeemer>> {
        let UnBuiltTransaction { actions } = unbuilt_tx;
        let (inputs, outputs, redeemers, scripts) = self.handle_actions(actions)?;

        Ok(Transaction {
            inputs,
            outputs,
            redeemers,
            scripts,
        })
    }
}

fn add_to_map(h_map: &mut HashMap<Policy, u64>, policy: Policy, amount: u64) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}

fn nested_map_to_vecs(
    nested_map: HashMap<Address, RefCell<HashMap<Policy, u64>>>,
) -> Vec<(Address, Vec<(Policy, u64)>)> {
    nested_map
        .into_iter()
        .map(|(addr, h_map)| (addr, h_map.into_inner().into_iter().collect()))
        .collect()
}

fn add_amount_to_nested_map(
    output_map: &mut HashMap<Address, RefCell<HashMap<Policy, u64>>>,
    amount: u64,
    owner: &Address,
    policy: &Policy,
) {
    if let Some(h_map) = output_map.get(owner) {
        let mut inner = h_map.borrow_mut();
        let mut new_total = amount;
        if let Some(total) = inner.get(policy) {
            new_total += total;
        }
        inner.insert(policy.clone(), new_total);
    } else {
        let mut new_map = HashMap::new();
        new_map.insert(policy.clone(), amount);
        output_map.insert(owner.clone(), RefCell::new(new_map));
    }
}

pub fn can_spend_inputs<
    Datum: Clone + PartialEq + Debug,
    Redeemer: Clone + PartialEq + Eq + Hash,
>(
    tx: &Transaction<Datum, Redeemer>,
    signer: Address,
) -> Result<()> {
    let ctx = TxContext { signer };
    for input in &tx.inputs {
        match input {
            Output::Wallet { .. } => {} // TODO: Make sure not spending other's outputs
            Output::Validator { owner, datum, .. } => {
                let script = tx.scripts.get(owner).ok_or(Error::TxORecord(
                    TxORecordError::FailedToRetrieveScriptFor(owner.to_owned()),
                ))?;
                let (_, redeemer) =
                    tx.redeemers
                        .iter()
                        .find(|(utxo, _)| utxo == input)
                        .ok_or(Error::TxORecord(
                            TxORecordError::FailedToRetrieveRedeemersFor(owner.to_owned()),
                        ))?;

                script
                    .execute(datum.clone(), redeemer.clone(), ctx.clone())
                    .map_err(|e| Error::ValidatorCode(e))?;
            }
        }
    }
    Ok(())
}
