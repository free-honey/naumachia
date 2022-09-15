use crate::output::OutputId;
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
    Transaction, TxActions,
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};
use uuid::Uuid;

mod nested_value_map;
pub mod selection;
pub mod tx_checks;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Backend<Datum, Redeemer, LC>
where
    Redeemer: Clone + Eq,
    LC: LedgerClient<Datum, Redeemer>,
{
    // TODO: Make fields private
    pub _datum: PhantomData<Datum>,
    pub _redeemer: PhantomData<Redeemer>,
    pub ledger_client: LC,
}

impl<Datum, Redeemer, LC> Backend<Datum, Redeemer, LC>
where
    Datum: Clone + Eq + Debug,
    Redeemer: Clone + Eq + Hash,
    LC: LedgerClient<Datum, Redeemer>,
{
    pub fn new(txo_record: LC) -> Self {
        Backend {
            _datum: PhantomData::default(),
            _redeemer: PhantomData::default(),
            ledger_client: txo_record,
        }
    }

    pub async fn process(&self, u_tx: TxActions<Datum, Redeemer>) -> Result<()> {
        let tx = self.build(u_tx).await?;
        can_spend_inputs(&tx, self.signer().await?.clone())?;
        can_mint_tokens(&tx, self.ledger_client.signer().await?)?;
        self.ledger_client.issue(tx).await?;
        Ok(())
    }

    pub fn ledger_client(&self) -> &LC {
        &self.ledger_client
    }

    pub async fn signer(&self) -> Result<&Address> {
        let addr = self.ledger_client.signer().await?;
        Ok(addr)
    }

    async fn handle_actions(
        &self,
        actions: Vec<Action<Datum, Redeemer>>,
    ) -> Result<Transaction<Datum, Redeemer>> {
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting: HashMap<Address, Values> = HashMap::new();
        let mut script_inputs: Vec<Output<Datum>> = Vec::new();
        let mut specific_outputs: Vec<Output<Datum>> = Vec::new();

        let mut redeemers = Vec::new();
        let mut validators = HashMap::new();
        let mut policies: HashMap<PolicyId, Box<dyn MintingPolicy>> = HashMap::new();
        let mut new_utxo_index = 0; // TODO: This feels cheap and error prone
        for action in actions {
            match action {
                Action::Transfer {
                    amount,
                    recipient,
                    policy_id: policy,
                } => {
                    add_amount_to_nested_map(&mut min_output_values, amount, &recipient, &policy);
                }
                Action::Mint {
                    amount,
                    recipient,
                    policy,
                } => {
                    let policy_id = policy.id();
                    if let Some(values) = minting.remove(&recipient) {
                        let mut new_values = values;
                        new_values.add_one_value(&policy_id, amount);
                        minting.insert(recipient, new_values);
                    } else {
                        let mut new_values = Values::default();
                        new_values.add_one_value(&policy_id, amount);
                        minting.insert(recipient, new_values);
                    }
                    // minting.add_one_value(&policy_id, amount);
                    policies.insert(policy.id(), policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    let tx_hash = Uuid::new_v4().to_string(); // TODO: This should be done by the TxORecord impl or something
                    let index = new_utxo_index;
                    new_utxo_index += 1;
                    let id = OutputId::new(tx_hash, index);
                    let owner = address;
                    let output = Output::Validator {
                        // TODO: This should happen later
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

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = self.create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        let tx = Transaction {
            script_inputs,
            outputs,
            redeemers,
            validators,
            minting,
            policies,
        };
        Ok(tx)
    }

    // TODO: This should be done by the LedgerClient
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
                // Very wrong
                let tx_hash = Uuid::new_v4().to_string();
                let index = 0;
                Output::new_wallet(tx_hash, index, owner, values)
            })
            .collect();
        Ok(outputs)
    }

    async fn build(
        &self,
        unbuilt_tx: TxActions<Datum, Redeemer>,
    ) -> Result<Transaction<Datum, Redeemer>> {
        let TxActions { actions } = unbuilt_tx;
        self.handle_actions(actions).await
        // TODO: Calculate fees and then rebuild tx
    }
}
