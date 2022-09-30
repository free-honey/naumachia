use crate::transaction::nested_value_map::{add_amount_to_nested_map, nested_map_to_vecs};
use crate::{
    address::{Address, PolicyId},
    backend::RedemptionDetails,
    error::*,
    output::{Output, UnbuiltOutput},
    scripts::{MintingPolicy, ValidatorCode},
    values::Values,
};
use std::cell::RefCell;
use std::collections::HashMap;

mod nested_value_map;

pub enum Action<Datum, Redeemer> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy_id: PolicyId,
    },
    // TODO: Support sending to script address
    Mint {
        amount: u64,
        recipient: Address,
        policy: Box<dyn MintingPolicy>,
    },
    InitScript {
        datum: Datum,
        values: Values,
        address: Address,
    },
    RedeemScriptOutput {
        output: Output<Datum>,
        redeemer: Redeemer,
        script: Box<dyn ValidatorCode<Datum, Redeemer>>, // Is there a way to do this without `dyn`?
    },
}

pub struct TxActions<Datum, Redeemer> {
    pub actions: Vec<Action<Datum, Redeemer>>,
}

impl<Datum, Redeemer> Default for TxActions<Datum, Redeemer> {
    fn default() -> Self {
        TxActions {
            actions: Vec::new(),
        }
    }
}

impl<Datum: Clone, Redeemer> TxActions<Datum, Redeemer> {
    pub fn with_transfer(mut self, amount: u64, recipient: Address, policy_id: PolicyId) -> Self {
        let action = Action::Transfer {
            amount,
            recipient,
            policy_id,
        };
        self.actions.push(action);
        self
    }

    pub fn with_mint(
        mut self,
        amount: u64,
        recipient: &Address,
        policy: Box<dyn MintingPolicy>,
    ) -> Self {
        let action = Action::Mint {
            amount,
            recipient: recipient.clone(),
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_script_init(mut self, datum: Datum, values: Values, address: Address) -> Self {
        let action = Action::InitScript {
            datum,
            values,
            address,
        };
        self.actions.push(action);
        self
    }

    // TODO: This can prolly just take the Output ID
    pub fn with_script_redeem(
        mut self,
        output: Output<Datum>,
        redeemer: Redeemer,
        script: Box<dyn ValidatorCode<Datum, Redeemer>>,
    ) -> Self {
        let action = Action::RedeemScriptOutput {
            output,
            redeemer,
            script,
        };
        self.actions.push(action);
        self
    }

    pub fn to_unbuilt_tx(self) -> Result<UnbuiltTransaction<Datum, Redeemer>> {
        let TxActions { actions } = self;
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting = Values::default();
        let mut script_inputs: Vec<RedemptionDetails<Datum, Redeemer>> = Vec::new();
        let mut specific_outputs: Vec<UnbuiltOutput<Datum>> = Vec::new();

        let mut policies: HashMap<PolicyId, Box<dyn MintingPolicy>> = HashMap::new();
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
                    minting.add_one_value(&policy_id, amount);
                    add_amount_to_nested_map(
                        &mut min_output_values,
                        amount,
                        &recipient,
                        &policy_id,
                    );
                    policies.insert(policy.id(), policy);
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    let owner = address;
                    let output = UnbuiltOutput::Validator {
                        script_address: owner,
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
                    script_inputs.push((output.clone(), redeemer, script));
                }
            }
        }

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        let tx = UnbuiltTransaction {
            script_inputs,
            unbuilt_outputs: outputs,
            minting,
            policies,
        };
        Ok(tx)
    }
}

fn create_outputs_for<Datum>(
    values: Vec<(Address, Vec<(PolicyId, u64)>)>,
) -> Result<Vec<UnbuiltOutput<Datum>>> {
    let outputs = values
        .into_iter()
        .map(|(owner, val_vec)| {
            let values = val_vec
                .iter()
                .fold(Values::default(), |mut acc, (policy, amt)| {
                    acc.add_one_value(policy, *amt);
                    acc
                });
            UnbuiltOutput::new_wallet(owner, values)
        })
        .collect();
    Ok(outputs)
}

pub struct UnbuiltTransaction<Datum, Redeemer> {
    pub script_inputs: Vec<RedemptionDetails<Datum, Redeemer>>,
    pub unbuilt_outputs: Vec<UnbuiltOutput<Datum>>,
    pub minting: Values,
    pub policies: HashMap<PolicyId, Box<dyn MintingPolicy>>,
}

impl<Datum, Redeemer> UnbuiltTransaction<Datum, Redeemer> {
    pub fn unbuilt_outputs(&self) -> &Vec<UnbuiltOutput<Datum>> {
        &self.unbuilt_outputs
    }

    pub fn script_inputs(&self) -> &Vec<RedemptionDetails<Datum, Redeemer>> {
        &self.script_inputs
    }
}

#[derive(Debug)]
pub struct TxId(String);

impl TxId {
    pub fn new(id_str: &str) -> Self {
        TxId(id_str.to_string())
    }

    pub fn as_str(&self) -> String {
        self.0.clone()
    }
}
