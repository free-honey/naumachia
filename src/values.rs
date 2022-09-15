use crate::{
    error::{Error, Result},
    output::Output,
    PolicyId,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

#[serde_with::serde_as]
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize, Default)]
pub struct Values {
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    values: HashMap<PolicyId, u64>,
}

impl Values {
    pub fn from_outputs<D>(outputs: &[Output<D>]) -> Self {
        outputs.iter().fold(Values::default(), |mut acc, output| {
            acc.add_values(output.values());
            acc
        })
    }

    pub fn try_subtract(&self, other: &Values) -> Result<Option<Values>> {
        let mut remainders = Vec::new();
        let mut mine_cloned = self.values.clone();
        let mut there_is_a_difference = false;
        if !other.is_empty() {
            for (policy, amt) in other.as_iter() {
                if let Some(available) = mine_cloned.remove(policy) {
                    match amt.cmp(&available) {
                        Ordering::Less => {
                            let remaining = available - amt;
                            remainders.push((policy.clone(), remaining));
                            there_is_a_difference = true;
                        }
                        Ordering::Greater => {
                            return Err(Error::InsufficientAmountOf(policy.to_owned()))
                        }
                        _ => {}
                    }
                } else {
                    return Err(Error::InsufficientAmountOf(policy.to_owned()));
                }
            }
        } else {
            there_is_a_difference = true; // We just keep what we started with
        }

        let other_remainders: Vec<_> = mine_cloned.into_iter().collect();
        remainders.extend(other_remainders);

        let values = remainders.into_iter().collect();
        if there_is_a_difference {
            let difference = Values { values };
            Ok(Some(difference))
        } else {
            Ok(None)
        }
    }

    pub fn add_one_value(&mut self, policy: &PolicyId, amount: u64) {
        add_to_map(&mut self.values, policy.clone(), amount)
    }

    pub fn add_values(&mut self, values: &Values) {
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

    pub fn get(&self, policy: &PolicyId) -> Option<u64> {
        self.values.get(policy).copied()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn add_to_map(h_map: &mut HashMap<PolicyId, u64>, policy: PolicyId, amount: u64) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}
