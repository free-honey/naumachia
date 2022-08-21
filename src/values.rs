use crate::backend::add_to_map;
use crate::error::{Error, Result};
use crate::output::Output;
use crate::PolicyId;
use std::collections::HashMap;
use std::ops::{Add, Sub};

#[derive(Default, Clone)]
pub struct Values {
    values: HashMap<PolicyId, u64>,
}

impl Values {
    pub fn from_outputs<D>(outputs: &[Output<D>]) -> Self {
        let mut values = HashMap::new();
        outputs
            .clone()
            .into_iter()
            .flat_map(|o| o.values().clone().into_iter().collect::<Vec<_>>())
            .for_each(|(policy, amount)| {
                add_to_map(&mut values, policy, amount);
            });
        Values { values }
    }

    pub fn add_output_value<D>(&mut self, output: &Output<D>) {
        todo!()
    }

    pub fn try_subtract(&self, other: &Values) -> Result<Values> {
        // TODO: make more efficient
        let mut remainders = Vec::new();
        let mut mine_cloned = self.values.clone();
        for (policy, amt) in other.vec().iter() {
            if let Some(available) = mine_cloned.remove(policy) {
                if amt <= &available {
                    let remaining = available - amt;
                    remainders.push((policy.clone(), remaining));
                } else {
                    return Err(Error::InsufficientAmountOf(policy.to_owned()));
                }
            } else {
                return Err(Error::InsufficientAmountOf(policy.to_owned()));
            }
        }
        let other_remainders: Vec<_> = mine_cloned.into_iter().collect();
        remainders.extend(other_remainders);

        let values = remainders.into_iter().collect();
        let difference = Values { values };
        Ok(difference)
    }

    pub fn add_one_value(&mut self, policy: &PolicyId, amount: u64) {
        add_to_map(&mut self.values, policy.clone(), amount)
    }
    //
    // pub fn remove_one_policy(&mut self, policy: &PolicyId) -> Option<u64> {
    //     self.values.remove(policy)
    // }

    pub fn add_values(&mut self, values: &Values) {
        // TODO: make more efficient
        for (policy, amt) in values.vec().iter() {
            self.add_one_value(policy, *amt)
        }
    }

    pub fn vec(&self) -> Vec<(PolicyId, u64)> {
        self.values.clone().into_iter().collect()
    }
}

impl Add for Values {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
