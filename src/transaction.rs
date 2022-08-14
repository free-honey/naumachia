use crate::{
    address::{Address, PolicyId},
    output::Output,
    scripts::{MintingPolicy, ValidatorCode},
};

use std::collections::HashMap;

pub enum Action<Datum, Redeemer> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy_id: PolicyId,
    },
    Mint {
        amount: u64,
        recipient: Address,
        policy: Box<dyn MintingPolicy>,
    },
    InitScript {
        datum: Datum,
        values: HashMap<PolicyId, u64>,
        address: Address,
    },
    RedeemScriptOutput {
        output: Output<Datum>,
        redeemer: Redeemer,
        script: Box<dyn ValidatorCode<Datum, Redeemer>>, // Is there a way to do this without `dyn`?
    },
}

pub struct UnBuiltTransaction<Datum, Redeemer> {
    pub actions: Vec<Action<Datum, Redeemer>>,
}

impl<Datum, Redeemer> Default for UnBuiltTransaction<Datum, Redeemer> {
    fn default() -> Self {
        UnBuiltTransaction {
            actions: Vec::new(),
        }
    }
}

impl<Datum, Redeemer> UnBuiltTransaction<Datum, Redeemer> {
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

    pub fn with_script_init(
        mut self,
        datum: Datum,
        values: HashMap<PolicyId, u64>,
        address: Address,
    ) -> Self {
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
}

pub struct Transaction<Datum, Redeemer> {
    pub inputs: Vec<Output<Datum>>,
    pub outputs: Vec<Output<Datum>>,
    pub redeemers: Vec<(Output<Datum>, Redeemer)>,
    pub scripts: HashMap<Address, Box<dyn ValidatorCode<Datum, Redeemer>>>,
    pub minting: HashMap<PolicyId, u64>,
    pub policies: HashMap<Address, Box<dyn MintingPolicy>>,
}

impl<Datum, Redeemer: Clone + PartialEq + Eq> Transaction<Datum, Redeemer> {
    pub fn outputs(&self) -> &Vec<Output<Datum>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Vec<Output<Datum>> {
        &self.inputs
    }
}
