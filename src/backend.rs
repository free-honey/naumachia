use crate::address::ValidAddress;
use crate::{
    address::PolicyId,
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
pub struct Backend<Address, Datum, Redeemer, LC>
where
    Redeemer: Clone + Eq,
    LC: LedgerClient<Datum, Redeemer>,
{
    // TODO: Make fields private
    pub _address: PhantomData<Address>,
    pub _datum: PhantomData<Datum>,
    pub _redeemer: PhantomData<Redeemer>,
    pub ledger_client: LC,
}

impl<Address, Datum, Redeemer, LC> Backend<Address, Datum, Redeemer, LC>
where
    Address: ValidAddress,
    Datum: Clone + Eq + Debug,
    Redeemer: Clone + Eq + Hash,
    LC: LedgerClient<Datum, Redeemer, Address = Address>,
{
    pub fn new(ledger_client: LC) -> Self {
        Backend {
            _address: PhantomData::default(),
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            ledger_client,
        }
    }

    pub fn process(&self, u_tx: UnBuiltTransaction<Address, Datum, Redeemer>) -> Result<()> {
        let tx = self.build(u_tx)?;
        can_spend_inputs(&tx, self.signer().clone())?;
        can_mint_tokens(&tx, &self.ledger_client.signer())?;
        self.ledger_client.issue(tx)?;
        Ok(())
    }

    pub fn txo_record(&self) -> &LC {
        &self.ledger_client
    }

    pub fn signer(&self) -> &Address {
        self.ledger_client.signer()
    }

    // TODO: Remove allow
    #[allow(clippy::type_complexity)]
    fn handle_actions(
        &self,
        actions: Vec<Action<Address, Datum, Redeemer>>,
    ) -> Result<Transaction<Address, Datum, Redeemer>> {
        let mut min_input_values = Values::default();
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting = Values::default();
        let mut script_inputs: Vec<Output<Address, Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Address, Datum>> = Vec::new();

        let mut redeemers = Vec::new();
        let mut validators = HashMap::new();
        let mut policies: HashMap<PolicyId, Box<dyn MintingPolicy<Address>>> = HashMap::new();
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
                    let policy_id = policy.id();
                    add_amount_to_nested_map(
                        &mut min_output_values,
                        amount,
                        &recipient,
                        &policy_id,
                    );
                    minting.add_one_value(&policy_id, amount);
                    policies.insert(policy.id(), policy);
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
        let (inputs, remainders) = self.select_inputs_for_one(
            &self.ledger_client.signer(),
            &min_input_values,
            script_inputs,
        )?;

        // TODO: Dedupe
        let mut new_values = remainders;
        if let Some(values) = min_output_values.get(&self.ledger_client.signer()) {
            new_values.add_values(&values.borrow());
        }
        min_output_values.insert(
            self.ledger_client.signer().clone(),
            RefCell::new(new_values),
        );

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
        script_inputs: Vec<Output<Address, Datum>>,
    ) -> Result<(Vec<Output<Address, Datum>>, Values)> {
        let mut all_available_outputs = self.ledger_client.outputs_at_address(address);
        all_available_outputs.extend(script_inputs);
        let address_values = Values::from_outputs(&all_available_outputs);

        let spending_outputs = all_available_outputs;
        let remainders = address_values.try_subtract(spending_values)?;
        Ok((spending_outputs, remainders))
    }

    fn create_outputs_for(
        &self,
        values: Vec<(Address, Vec<(PolicyId, u64)>)>,
    ) -> Result<Vec<Output<Address, Datum>>> {
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
        unbuilt_tx: UnBuiltTransaction<Address, Datum, Redeemer>,
    ) -> Result<Transaction<Address, Datum, Redeemer>> {
        let UnBuiltTransaction { actions } = unbuilt_tx;
        self.handle_actions(actions)
        // TODO: Calculate fees and then rebuild tx
    }
}
