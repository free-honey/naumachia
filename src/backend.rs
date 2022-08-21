use crate::scripts::MintingPolicy;
use crate::values::Values;
use crate::{
    address::{Address, PolicyId},
    error::Error,
    error::Result,
    ledger_client::LedgerClient,
    output::Output,
    scripts::{TxContext, ValidatorCode},
    transaction::Action,
    Transaction, UnBuiltTransaction,
};
use std::borrow::Borrow;
use std::cmp::min;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

pub mod selection;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Backend<Datum, Redeemer: Clone + Eq, Record: LedgerClient<Datum, Redeemer>> {
    // TODO: Make fields private
    pub _datum: PhantomData<Datum>,
    pub _redeemer: PhantomData<Redeemer>,
    pub txo_record: Record,
}

impl<Datum, Redeemer, Record> Backend<Datum, Redeemer, Record>
where
    Datum: Clone + Eq + Debug,
    Redeemer: Clone + Eq + Hash,
    Record: LedgerClient<Datum, Redeemer>,
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
        can_spend_inputs(&tx, self.signer().clone())?;
        can_mint_tokens(&tx, self.txo_record.signer())?;
        self.txo_record.issue(tx)?;
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
        HashMap<PolicyId, u64>,
        HashMap<Address, Box<dyn MintingPolicy>>,
    )> {
        let mut min_input_values = Values::default();
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting: HashMap<PolicyId, u64> = HashMap::new();
        let mut script_inputs: Vec<Output<Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Datum>> = Vec::new();

        let mut redeemers = Vec::new();
        let mut validator_scripts = HashMap::new();
        let mut policy_scripts: HashMap<Address, Box<dyn MintingPolicy>> = HashMap::new();
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy_id: policy,
                } => {
                    // Input
                    // add_to_map(&mut min_input_values, policy.clone(), amount);
                    min_input_values.add_one_value(&policy, amount);

                    // Output
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::Mint {
                    amount,
                    recipient,
                    policy,
                } => {
                    let policy_id = Some(policy.address());
                    add_amount_to_nested_map(
                        &mut min_output_values,
                        amount,
                        &recipient,
                        &policy_id,
                    );
                    add_to_map(&mut minting, policy_id.clone(), amount);
                    policy_scripts.insert(policy.address(), policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    for (policy, amount) in values.iter() {
                        // add_to_map(&mut min_input_values, policy.clone(), *amount);
                        min_input_values.add_one_value(&policy, *amount);
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
                    validator_scripts.insert(script_address, script);
                }
            }
        }
        // inputs
        let (inputs, remainders) =
            self.select_inputs_for_one(self.txo_record.signer(), &min_input_values, script_inputs)?;

        // outputs
        // remainders.iter().for_each(|(amt, policy)| {
        //     add_amount_to_nested_map(
        //         &mut min_output_values,
        //         *amt,
        //         &self.txo_record.signer(),
        //         policy,
        //     )
        // });

        // TODO: Dedupe
        let mut new_values = remainders;
        if let Some(values) = min_output_values.get(&self.txo_record.signer()) {
            new_values.add_values(&values.borrow());
        }
        min_output_values.insert(self.txo_record.signer().clone(), RefCell::new(new_values));

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = self.create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        Ok((
            inputs,
            outputs,
            redeemers,
            validator_scripts,
            minting,
            policy_scripts,
        ))
    }

    // LOL Super Naive Solution, just select ALL inputs!
    // TODO: Use Random Improve prolly: https://cips.cardano.org/cips/cip2/
    //       but this is _good_enough_ for tests.
    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn select_inputs_for_one(
        &self,
        address: &Address,
        // values: &HashMap<PolicyId, u64>,
        spending_values: &Values,
        script_inputs: Vec<Output<Datum>>,
    ) -> Result<(Vec<Output<Datum>>, Values)> {
        let mut all_available_outputs = self.txo_record.outputs_at_address(address);
        all_available_outputs.extend(script_inputs);
        let address_values = Values::from_outputs(&all_available_outputs);

        let spending_outputs = all_available_outputs;
        let remainders = address_values.try_subtract(spending_values)?;
        Ok((spending_outputs, remainders))
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(PolicyId, u64)>)>,
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
        let (inputs, outputs, redeemers, scripts, minting, policies) =
            self.handle_actions(actions)?;
        // TODO: Calculate fees and then rebuild tx
        Ok(Transaction {
            inputs,
            outputs,
            redeemers,
            scripts,
            minting,
            policies,
        })
    }
}

pub fn add_to_map(h_map: &mut HashMap<PolicyId, u64>, policy: PolicyId, amount: u64) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}

fn nested_map_to_vecs(
    nested_map: HashMap<Address, RefCell<Values>>,
) -> Vec<(Address, Vec<(PolicyId, u64)>)> {
    nested_map
        .into_iter()
        .map(|(addr, values)| (addr, values.borrow().vec()))
        .collect()
}

fn add_amount_to_nested_map(
    output_map: &mut HashMap<Address, RefCell<Values>>,
    amount: u64,
    owner: &Address,
    policy_id: &PolicyId,
) {
    if let Some(values) = output_map.get(owner) {
        let mut inner = values.borrow_mut();
        // let mut new_total = amount;
        // if let Some(total) = inner.get(policy_id) {
        //     new_total += total;
        // }
        // inner.insert(policy_id.clone(), new_total);
        inner.add_one_value(policy_id, amount);
    } else {
        let mut new_values = Values::default();
        new_values.add_one_value(&policy_id, amount);
        output_map.insert(owner.clone(), RefCell::new(new_values));
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
                let script = tx
                    .scripts
                    .get(owner)
                    .ok_or_else(|| Error::FailedToRetrieveScriptFor(owner.to_owned()))?;
                let (_, redeemer) = tx
                    .redeemers
                    .iter()
                    .find(|(utxo, _)| utxo == input)
                    .ok_or_else(|| Error::FailedToRetrieveRedeemerFor(owner.to_owned()))?;

                script.execute(datum.clone(), redeemer.clone(), ctx.clone())?;
            }
        }
    }
    Ok(())
}

pub fn can_mint_tokens<Datum, Redeemer>(
    tx: &Transaction<Datum, Redeemer>,
    signer: &Address,
) -> Result<()> {
    let ctx = TxContext {
        signer: signer.clone(),
    };
    for (id, _) in tx.minting.iter() {
        if let Some(address) = id {
            if let Some(policy) = tx.policies.get(address) {
                policy.execute(ctx.clone())?;
            } else {
                return Err(Error::FailedToRetrieveScriptFor(address.clone()));
            }
        } else {
            return Err(Error::ImpossibleToMintADA);
        }
    }
    Ok(())
}
