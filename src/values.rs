use crate::{
    error::{
        Error,
        Result,
    },
    output::Output,
    PolicyId,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
};

/// Domain representation of value on the Cardano blockchain
#[serde_with::serde_as]
#[derive(Clone, PartialEq, Debug, Eq, Deserialize, Serialize, Default)]
pub struct Values {
    #[serde_as(as = "HashMap<serde_with::json::JsonString, _>")]
    values: HashMap<PolicyId, u64>,
}

impl Values {
    /// Construct a `Values` from a list of `Output`s
    pub fn from_outputs<D>(outputs: &[Output<D>]) -> Self {
        outputs.iter().fold(Values::default(), |mut acc, output| {
            acc.add_values(output.values());
            acc
        })
    }

    /// Try to remove the `other` `Values` from `self`
    pub fn try_subtract(&self, other: &Values) -> Result<Values> {
        let mut remainders = Vec::new();
        let mut mine_cloned = self.values.clone();
        if !other.is_empty() {
            for (policy, amt) in other.as_iter() {
                if let Some(available) = mine_cloned.remove(policy) {
                    match amt.cmp(&available) {
                        Ordering::Less => {
                            let remaining = available - amt;
                            remainders.push((policy.clone(), remaining));
                        }
                        Ordering::Greater => {
                            return Err(Error::InsufficientAmountOf(policy.to_owned()))
                        }
                        _ => {
                            // no need to add anything to resulting values since the amount is 0
                        }
                    }
                } else {
                    return Err(Error::InsufficientAmountOf(policy.to_owned()));
                }
            }
        }

        let other_remainders: Vec<_> = mine_cloned.into_iter().collect();
        remainders.extend(other_remainders);

        let values = remainders.into_iter().collect();
        Ok(Values { values })
    }

    /// Add one value to the `self`
    pub fn add_one_value(&mut self, policy: &PolicyId, amount: u64) {
        add_to_map(&mut self.values, policy.clone(), amount)
    }

    /// Add a `Values` to the `self`
    pub fn add_values(&mut self, values: &Values) {
        for (policy, amt) in values.as_iter() {
            self.add_one_value(policy, *amt)
        }
    }

    /// Convert the `Values` to an iterator of [`PolicyId`]s and amounts
    pub fn as_iter(&self) -> std::collections::hash_map::Iter<'_, PolicyId, u64> {
        self.values.iter()
    }

    /// Convert the `Values` to a `Vec` of [`PolicyId`]s and amounts
    pub fn vec(&self) -> Vec<(PolicyId, u64)> {
        self.values.clone().into_iter().collect()
    }

    /// Get the amount for a given [`PolicyId`]
    pub fn get(&self, policy: &PolicyId) -> Option<u64> {
        self.values.get(policy).copied()
    }

    /// Remove all values for a given [`PolicyId`] and return the amount
    pub fn take(&mut self, policy: &PolicyId) -> Option<u64> {
        self.values.remove(policy)
    }

    /// Get the number of [`PolicyId`]s in the `Values`
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the `Values` is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub(crate) fn add_to_map(
    h_map: &mut HashMap<PolicyId, u64>,
    policy: PolicyId,
    amount: u64,
) {
    let mut new_total = amount;
    if let Some(total) = h_map.get(&policy) {
        new_total += total;
    }
    h_map.insert(policy.clone(), new_total);
}
