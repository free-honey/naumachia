use crate::error::{Error, Result};
use crate::output::Output;
use crate::PolicyId;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct Values {
    values: HashMap<PolicyId, u64>,
}

impl Values {
    pub fn from_outputs<D>(outputs: &[Output<D>]) -> Self {
        let mut values = HashMap::new();
        outputs
            .iter()
            .flat_map(|o| o.values().clone().into_iter().collect::<Vec<_>>())
            .for_each(|(policy, amount)| {
                add_to_map(&mut values, policy, amount);
            });
        Values { values }
    }

    pub fn try_subtract(&self, other: &Values) -> Result<Values> {
        let mut remainders = Vec::new();
        let mut mine_cloned = self.values.clone();
        for (policy, amt) in other.as_iter() {
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

    pub fn add_values(&mut self, values: &Values) {
        // TODO: make more efficient
        for (policy, amt) in values.as_iter() {
            self.add_one_value(policy, *amt)
        }
    }

    pub fn as_iter(&self) -> std::collections::hash_map::Iter<'_, PolicyId, u64> {
        self.values.iter()
    }

    pub fn vec(&self) -> Vec<(PolicyId, u64)> {
        self.values.clone().into_iter().collect()
    }
}

pub fn add_to_map(h_map: &mut HashMap<PolicyId, u64>, policy: PolicyId, amount: u64) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}
