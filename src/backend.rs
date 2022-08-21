use crate::{
    address::{Address, PolicyId},
    backend::nested_value_map::{add_amount_to_nested_map, nested_map_to_vecs},
    backend::tx_checks::{can_mint_tokens, can_spend_inputs},
    error::Result,
    ledger_client::LedgerClient,
    output::Output,
    scripts::MintingPolicy,
    transaction::Action,
    values::Values,
    Transaction, UnBuiltTransaction,
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

mod nested_value_map;
pub mod selection;
pub mod tx_checks;

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
    ) -> Result<Transaction<Datum, Redeemer>> {
        let mut min_input_values = Values::default();
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting = Values::default();
        let mut script_inputs: Vec<Output<Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Datum>> = Vec::new();

        let mut redeemers = Vec::new();
        let mut validators = HashMap::new();
        let mut policies: HashMap<Address, Box<dyn MintingPolicy>> = HashMap::new();
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy_id: policy,
                } => {
                    // Input
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
                    minting.add_one_value(&policy_id, amount);
                    policies.insert(policy.address(), policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    for (policy, amount) in values.as_iter() {
                        min_input_values.add_one_value(policy, *amount);
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
                    validators.insert(script_address, script);
                }
            }
        }
        // inputs
        let (inputs, remainders) =
            self.select_inputs_for_one(self.txo_record.signer(), &min_input_values, script_inputs)?;

        // TODO: Dedupe
        let mut new_values = remainders;
        if let Some(values) = min_output_values.get(self.txo_record.signer()) {
            new_values.add_values(&values.borrow());
        }
        min_output_values.insert(self.txo_record.signer().clone(), RefCell::new(new_values));

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = self.create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        let tx = Transaction {
            inputs,
            outputs,
            redeemers,
            validators,
            minting,
            policies,
        };
        Ok(tx)
    }

    // LOL Super Naive Solution, just select ALL inputs!
    // TODO: Use Random Improve prolly: https://cips.cardano.org/cips/cip2/
    //       but this is _good_enough_ for tests.
    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn select_inputs_for_one(
        &self,
        address: &Address,
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
                let values = val_vec
                    .iter()
                    .fold(Values::default(), |mut acc, (policy, amt)| {
                        acc.add_one_value(policy, *amt);
                        acc
                    });
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
        self.handle_actions(actions)
        // TODO: Calculate fees and then rebuild tx
    }
}
