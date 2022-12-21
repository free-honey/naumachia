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
use std::fmt::Debug;

pub(crate) mod nested_value_map;

pub enum Action<Datum, Redeemer> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy_id: PolicyId,
    },
    // TODO: Support sending to script address
    Mint {
        amount: u64,
        asset_name: Option<String>,
        redeemer: Redeemer,
        policy: Box<dyn MintingPolicy<Redeemer>>,
        recipient: Address,
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
    SpecificInput {
        input: Output<Datum>,
    },
}

// TODO: Maybe we should make V1 and V2 TxActions be completely different types,
//   since they have different options e.g. inline datum etc
pub struct TxActions<Datum, Redeemer> {
    pub script_version: TransactionVersion,
    pub actions: Vec<Action<Datum, Redeemer>>,
}

impl<Datum, Redeemer> TxActions<Datum, Redeemer> {
    pub fn v1() -> Self {
        TxActions {
            script_version: TransactionVersion::V1,
            actions: Vec::new(),
        }
    }

    pub fn v2() -> Self {
        TxActions {
            script_version: TransactionVersion::V2,
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

    // TODO: I don't know if that recipient makes any sense. It can just be included in the default
    //   outputs or specified outputs (anything unspecified will just go to creator)
    pub fn with_mint(
        mut self,
        amount: u64,
        asset_name: Option<String>,
        recipient: &Address,
        redeemer: Redeemer,
        policy: Box<dyn MintingPolicy<Redeemer>>,
    ) -> Self {
        let action = Action::Mint {
            amount,
            asset_name,
            redeemer,
            policy,
            recipient: recipient.clone(),
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

    pub fn with_specific_input(mut self, input: Output<Datum>) -> Self {
        let action = Action::SpecificInput { input };
        self.actions.push(action);
        self
    }

    pub fn to_unbuilt_tx(self) -> Result<UnbuiltTransaction<Datum, Redeemer>> {
        let TxActions {
            script_version,
            actions,
        } = self;
        let mut min_output_values: HashMap<Address, RefCell<Values>> = HashMap::new();
        let mut minting = Vec::new();
        let mut script_inputs: Vec<RedemptionDetails<Datum, Redeemer>> = Vec::new();
        let mut specific_outputs: Vec<UnbuiltOutput<Datum>> = Vec::new();
        let mut specific_wallet_inputs: Vec<Output<Datum>> = Vec::new();

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
                    asset_name,
                    redeemer,
                    policy,
                    recipient,
                } => {
                    let id = policy.id()?;
                    let policy_id = PolicyId::native_token(&id, &asset_name);
                    minting.push((amount, asset_name, redeemer, policy));
                    add_amount_to_nested_map(
                        &mut min_output_values,
                        amount,
                        &recipient,
                        &policy_id,
                    );
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
                Action::SpecificInput { input } => specific_wallet_inputs.push(input),
            }
        }

        let out_vecs = nested_map_to_vecs(min_output_values);
        let mut outputs = create_outputs_for(out_vecs)?;
        outputs.extend(specific_outputs);

        let tx = UnbuiltTransaction {
            script_version,
            script_inputs,
            unbuilt_outputs: outputs,
            minting,
            specific_wallet_inputs,
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

#[derive(Clone)]
pub enum TransactionVersion {
    V1,
    V2,
}

pub struct UnbuiltTransaction<Datum, Redeemer> {
    pub script_version: TransactionVersion,
    pub script_inputs: Vec<RedemptionDetails<Datum, Redeemer>>,
    pub unbuilt_outputs: Vec<UnbuiltOutput<Datum>>,
    #[allow(clippy::type_complexity)]
    pub minting: Vec<(
        u64,
        Option<String>,
        Redeemer,
        Box<dyn MintingPolicy<Redeemer>>,
    )>,
    pub specific_wallet_inputs: Vec<Output<Datum>>,
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
