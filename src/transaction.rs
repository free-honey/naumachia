use crate::transaction::nested_value_map::{add_amount_to_nested_map, nested_map_to_vecs};
use crate::{
    backend::RedemptionDetails,
    error::*,
    output::{Output, UnbuiltOutput},
    policy_id::PolicyId,
    scripts::{MintingPolicy, Validator},
    values::Values,
};
use pallas_addresses::Address;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;

pub(crate) mod nested_value_map;

/// Declarative constraints for specifying what a transaction should do.
pub enum Action<Datum, Redeemer> {
    /// Specify a transfer of `amount` or `policy_id` to `recipient`
    Transfer {
        /// Amount to transfer
        amount: u64,
        /// Recipient of the transfer
        recipient: Address,
        /// Policy ID of the asset to transfer
        policy_id: PolicyId,
    },
    /// Specify a minting of `amount` of `asset_name` to `redeemer` with `policy`
    Mint {
        /// Amount to mint
        amount: u64,
        /// Name of the asset to mint
        asset_name: Option<String>,
        /// Redeemer used with the minting policy
        redeemer: Redeemer,
        /// Minting policy
        policy: Box<dyn MintingPolicy<Redeemer>>,
    },
    /// Specify a script a value that will be locked at a script `address` with `datum`
    InitScript {
        /// Datum to lock
        datum: Datum,
        /// Values to lock
        values: Values,
        /// Address to lock at
        address: Address,
    },
    /// Specify a script output that will be redeemed with `redeemer` and `script`
    RedeemScriptOutput {
        /// Output to redeem that has attached datum
        output: Output<Datum>,
        /// Redeemer used with the validator
        redeemer: Redeemer,
        /// Validator used to validate the transaction
        script: Box<dyn Validator<Datum, Redeemer>>, // Is there a way to do this without `dyn`?
    },
    /// Specify a specific input to use in the transaction
    SpecificInput {
        /// Input to use
        input: Output<Datum>,
    },
}

// TODO: Maybe we should make V1 and V2 TxActions be completely different types,
//   since they have different options e.g. inline datum etc
/// Collection of declarative constraints for specifying what a transaction should do.
pub struct TxActions<Datum, Redeemer> {
    /// Version of the transaction
    pub script_version: TransactionVersion,
    /// Actions to be taken
    pub actions: Vec<Action<Datum, Redeemer>>,
    /// Valid range in seconds since the Unix epoch
    pub valid_range: Range,
}

impl<Datum, Redeemer> TxActions<Datum, Redeemer> {
    /// Constructor for a V1 TxActions
    pub fn v1() -> Self {
        TxActions {
            script_version: TransactionVersion::V1,
            actions: Vec::new(),
            valid_range: (None, None),
        }
    }

    /// Constructor for a V2 TxActions
    pub fn v2() -> Self {
        TxActions {
            script_version: TransactionVersion::V2,
            actions: Vec::new(),
            valid_range: (None, None),
        }
    }
}

impl<Datum: Clone, Redeemer> TxActions<Datum, Redeemer> {
    /// Add a transfer to the actions.
    /// This will transfer `amount` of `policy_id` to `recipient` without specifying specific inputs
    /// and outputs.
    pub fn with_transfer(mut self, amount: u64, recipient: Address, policy_id: PolicyId) -> Self {
        let action = Action::Transfer {
            amount,
            recipient,
            policy_id,
        };
        self.actions.push(action);
        self
    }

    /// Add a mint to the actions.
    /// This will mint `amount` of `asset_name` to `redeemer` with `policy` without specifying.
    /// The recipient isn't specified. Use other methods to specify the recipient.
    pub fn with_mint(
        mut self,
        amount: u64,
        asset_name: Option<String>,
        redeemer: Redeemer,
        policy: Box<dyn MintingPolicy<Redeemer>>,
    ) -> Self {
        let action = Action::Mint {
            amount,
            asset_name,
            redeemer,
            policy,
        };
        self.actions.push(action);
        self
    }

    /// Add a script init to the actions.
    /// This will lock the `values` at the `address` with the `datum`.
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
    /// Add a script redeem to the actions.
    /// This will redeem the `output` with the `redeemer` and use the `script` to validate the
    /// transaction.
    pub fn with_script_redeem(
        mut self,
        output: Output<Datum>,
        redeemer: Redeemer,
        script: Box<dyn Validator<Datum, Redeemer>>,
    ) -> Self {
        let action = Action::RedeemScriptOutput {
            output,
            redeemer,
            script,
        };
        self.actions.push(action);
        self
    }

    /// Add a specific input to the actions.
    /// **NOTE**: if you are using CML, this can break if your input is too small and you don't
    /// specify any specific OUTPUTs: https://github.com/MitchTurner/naumachia/issues/73
    pub fn with_specific_input(mut self, input: Output<Datum>) -> Self {
        let action = Action::SpecificInput { input };
        self.actions.push(action);
        self
    }

    /// Specify valid range in seconds since the Unix epoch
    pub fn with_valid_range_secs(mut self, lower: Option<i64>, upper: Option<i64>) -> Self {
        self.valid_range = (lower, upper);
        self
    }

    /// Convert the TxActions into an [`UnbuiltTransaction`] that can be consumed by a [`LedgerClient`]
    /// to submit a fully formed transaction.
    pub fn to_unbuilt_tx(self) -> Result<UnbuiltTransaction<Datum, Redeemer>> {
        let TxActions {
            script_version,
            actions,
            ..
        } = self;
        let mut min_output_values: HashMap<String, RefCell<Values>> = HashMap::new();
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
                } => {
                    minting.push((amount, asset_name, redeemer, policy));
                }
                Action::InitScript {
                    datum,
                    values,
                    address,
                } => {
                    let owner = address.to_bech32().expect("Already Validated");
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
            valid_range: self.valid_range,
        };
        Ok(tx)
    }
}

fn create_outputs_for<Datum>(
    values: Vec<(String, Vec<(PolicyId, u64)>)>,
) -> Result<Vec<UnbuiltOutput<Datum>>> {
    let outputs: Result<Vec<_>> = values
        .into_iter()
        .map(|(owner, val_vec)| {
            let values = val_vec
                .iter()
                .fold(Values::default(), |mut acc, (policy, amt)| {
                    acc.add_one_value(policy, *amt);
                    acc
                });
            let addr = Address::from_bech32(&owner)
                .map_err(|e| Error::Address(format!("Bad Bech32: {e:?}")))?;
            Ok(UnbuiltOutput::new_wallet(addr, values))
        })
        .collect();
    outputs
}

/// Version of the transaction
#[derive(Clone)]
#[non_exhaustive]
pub enum TransactionVersion {
    /// V1 transaction
    V1,
    /// V2 transaction
    V2,
}

/// Range of times in seconds since the Unix epoch
type Range = (Option<i64>, Option<i64>);

/// Unbuilt transaction that can be consumed by a [`LedgerClient`] to submit a fully formed transaction
pub struct UnbuiltTransaction<Datum, Redeemer> {
    /// Version of the transaction
    pub script_version: TransactionVersion,
    /// Script inputs to be redeemed
    pub script_inputs: Vec<RedemptionDetails<Datum, Redeemer>>,
    /// Outputs to be created
    pub unbuilt_outputs: Vec<UnbuiltOutput<Datum>>,
    #[allow(clippy::type_complexity)]
    /// Minting policies to be used
    pub minting: Vec<(
        u64,
        Option<String>,
        Redeemer,
        Box<dyn MintingPolicy<Redeemer>>,
    )>,
    /// Specific wallet inputs to be used
    pub specific_wallet_inputs: Vec<Output<Datum>>,
    /// Valid range in seconds since the Unix epoch
    pub valid_range: Range,
}

impl<Datum, Redeemer> UnbuiltTransaction<Datum, Redeemer> {
    /// Getter for the unbuilt outputs for the transaction
    pub fn unbuilt_outputs(&self) -> &Vec<UnbuiltOutput<Datum>> {
        &self.unbuilt_outputs
    }

    /// Getter for the script inputs for the transaction
    pub fn script_inputs(&self) -> &Vec<RedemptionDetails<Datum, Redeemer>> {
        &self.script_inputs
    }
}

/// The resulting transaction from a [`LedgerClient`] submission
#[derive(Debug)]
pub struct TxId(String);

impl TxId {
    /// Constructor for a TxId
    pub fn new(id_str: &str) -> Self {
        TxId(id_str.to_string())
    }

    /// String representation of the `TxId`
    pub fn as_str(&self) -> String {
        self.0.clone()
    }
}
