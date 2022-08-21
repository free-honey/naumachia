use crate::values::Values;
use crate::{Address, PolicyId};
use std::cell::RefCell;
use std::collections::HashMap;

pub(crate) fn nested_map_to_vecs(
    nested_map: HashMap<Address, RefCell<Values>>,
) -> Vec<(Address, Vec<(PolicyId, u64)>)> {
    nested_map
        .into_iter()
        .map(|(addr, values)| (addr, values.borrow().vec()))
        .collect()
}

pub(crate) fn add_amount_to_nested_map(
    output_map: &mut HashMap<Address, RefCell<Values>>,
    amount: u64,
    owner: &Address,
    policy_id: &PolicyId,
) {
    if let Some(values) = output_map.get(owner) {
        let mut inner = values.borrow_mut();
        inner.add_one_value(policy_id, amount);
    } else {
        let mut new_values = Values::default();
        new_values.add_one_value(policy_id, amount);
        output_map.insert(owner.clone(), RefCell::new(new_values));
    }
}
