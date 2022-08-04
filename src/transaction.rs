use crate::validator::ValidatorCode;
use crate::{
    address::{Address, Policy},
    output::Output,
};
use std::collections::HashMap;
use std::hash::Hash;

pub enum Action<Datum, Redeemer> {
    Transfer {
        amount: u64,
        recipient: Address,
        policy: Policy,
    },
    Mint {
        amount: u64,
        recipient: Address,
        policy: Policy,
    },
    InitScript {
        datum: Datum,
        values: HashMap<Policy, u64>,
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
    pub fn with_transfer(mut self, amount: u64, recipient: Address, policy: Policy) -> Self {
        let action = Action::Transfer {
            amount,
            recipient,
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_mint(mut self, amount: u64, recipient: Address, policy: Policy) -> Self {
        let action = Action::Mint {
            amount,
            recipient,
            policy,
        };
        self.actions.push(action);
        self
    }

    pub fn with_script_init(
        mut self,
        datum: Datum,
        values: HashMap<Policy, u64>,
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

#[derive(PartialEq, Debug)]
pub struct Transaction<Datum, Redeemer: Clone + PartialEq + Eq> {
    pub inputs: Vec<Output<Datum>>,
    pub outputs: Vec<Output<Datum>>,
    pub redeemers: Vec<(Output<Datum>, Redeemer)>,
}

impl<Datum, Redeemer: Clone + PartialEq + Eq> Transaction<Datum, Redeemer> {
    pub fn outputs(&self) -> &Vec<Output<Datum>> {
        &self.outputs
    }

    pub fn inputs(&self) -> &Vec<Output<Datum>> {
        &self.inputs
    }
}
