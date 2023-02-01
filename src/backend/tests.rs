#![allow(non_snake_case)]
use super::*;
use crate::PolicyId;
use proptest::prelude::*;
use proptest::test_runner::TestRng;

prop_compose! {
    fn arb_rng()(
        bytes: [u8; 32],
    ) -> TestRng {
        TestRng::from_seed(prop::test_runner::RngAlgorithm::ChaCha, &bytes)
    }
}

prop_compose! {
    fn arb_address()(
        addr: String
    ) -> Address {
        todo!();
        // Address::new(&addr)
    }
}

prop_compose! {
    fn arb_policy_id()(
        id: String,
    ) -> PolicyId {
        PolicyId::native_token(&id, &None)
    }
}

mod mint;
mod transfer;

// So IDE recognizes there are tests here
#[test]
fn empty() {}
