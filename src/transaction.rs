use crate::{
    address::PolicyId,
    output::Output,
    scripts::{MintingPolicy, ValidatorCode},
};

use crate::address::ValidAddress;
use crate::values::Values;
use std::collections::HashMap;

pub enum Action<Address, Datum, Redeemer> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy_id: PolicyId,
    },
    Mint {
        amount: u64,
        recipient: Address,
        policy: Box<dyn MintingPolicy<Address>>,
    },
    InitScript {
        datum: Datum,
        values: Values,
        address: Address,
    },
    RedeemScriptOutput {
        output: Output<Address, Datum>,
        redeemer: Redeemer,
        script: Box<dyn ValidatorCode<Address, Datum, Redeemer>>, // Is there a way to do this without `dyn`?
    },
}

pub struct UnBuiltTransaction<Address, Datum, Redeemer> {
    pub actions: Vec<Action<Address, Datum, Redeemer>>,
}

impl<Address, Datum, Redeemer> Default for UnBuiltTransaction<Address, Datum, Redeemer> {
    fn default() -> Self {
        UnBuiltTransaction {
            actions: Vec::new(),
        }
    }
}

impl<Address: ValidAddress, Datum, Redeemer> UnBuiltTransaction<Address, Datum, Redeemer> {
    pub fn with_transfer(mut self, amount: u64, recipient: &Address, policy_id: PolicyId) -> Self {
        let action: Action<Address, Datum, Redeemer> = Action::Transfer {
            amount,
            recipient: recipient.to_owned(),
            policy_id,
        };
        self.actions.push(action);
        self
    }

    pub fn with_mint(
        mut self,
        amount: u64,
        recipient: &Address,
        policy: Box<dyn MintingPolicy<Address>>,
    ) -> Self {
        let action = Action::Mint {
            amount,
            recipient: recipient.to_owned(),
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_script_init(mut self, datum: Datum, values: Values, address: Address) -> Self {
        let action = Action::InitScript {
            datum,
            values,
            address: address.to_owned(),
        };
        self.actions.push(action);
        self
    }

    // TODO: This can prolly just take the Output ID
    pub fn with_script_redeem(
        mut self,
        output: Output<Address, Datum>,
        redeemer: Redeemer,
        script: Box<dyn ValidatorCode<Address, Datum, Redeemer>>,
    ) -> Self {
        let action = Action::RedeemScriptOutput {
            output,
            redeemer,
            script,
        };
        self.actions.push(action);
        self
    }
}

pub struct Transaction<Address, Datum, Redeemer> {
    pub inputs: Vec<Output<Address, Datum>>,
    pub outputs: Vec<Output<Address, Datum>>,
    pub redeemers: Vec<(Output<Address, Datum>, Redeemer)>,
    pub validators: HashMap<Address, Box<dyn ValidatorCode<Address, Datum, Redeemer>>>,
    pub minting: Values,
    pub policies: HashMap<PolicyId, Box<dyn MintingPolicy<Address>>>,
}

impl<Address, Datum, Redeemer: Clone + PartialEq + Eq> Transaction<Address, Datum, Redeemer> {
    pub fn outputs(&self) -> &Vec<Output<Address, Datum>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Vec<Output<Address, Datum>> {
        &self.inputs
    }
}
