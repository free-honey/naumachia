use crate::{values::Values, PolicyId};
use pallas_addresses::Address;
use std::{cell::RefCell, collections::HashMap};

pub(crate) fn nested_map_to_vecs(
    nested_map: HashMap<String, RefCell<Values>>,
) -> Vec<(String, Vec<(PolicyId, u64)>)> {
    nested_map
        .into_iter()
        .map(|(addr, values)| (addr, values.borrow().vec()))
        .collect()
}

pub(crate) fn add_amount_to_nested_map(
    output_map: &mut HashMap<String, RefCell<Values>>,
    amount: u64,
    owner: &Address,
    policy_id: &PolicyId,
) {
    let owner_str = owner.to_bech32().expect("Already validated");
    if let Some(values) = output_map.get(&owner_str) {
        let mut inner = values.borrow_mut();
        inner.add_one_value(policy_id, amount);
    } else {
        let mut new_values = Values::default();
        new_values.add_one_value(policy_id, amount);
        output_map.insert(owner_str.clone(), RefCell::new(new_values));
    }
}
