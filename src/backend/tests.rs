use super::*;
use crate::backend::fake_backend::FakeRecord;
use crate::{address::ADA, backend::fake_backend::TestBackendsBuilder};
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
        Address::new(&addr)
    }
}

prop_compose! {
    fn arb_backend_with_enough()(
        min_amount: u32,
        mut rng in arb_rng(),
        their_utxo_count: u8,
        decoys in prop::collection::vec(arb_address().prop_filter("can't be alice or bob", |d| d != &Address::new("alice") && d != &Address::new("bob")), 0..10),
    ) -> (Address, Address, u64, Backend<(),(), FakeRecord<(),()>>, Vec<Address>) {
        let signer = Address::new("alice");
        let recipient = Address::new("bob");
        let mut total: u64 = 0;
        let mut builder = TestBackendsBuilder::<(), ()>::new(&signer);
        while total < min_amount as u64 {
            let new_amount = rng.gen_range(0..=min_amount);
            builder = builder.start_output(&signer).with_value(ADA, new_amount as u64).finish_output();
            for decoy in &decoys {
                let new_amount = rng.gen_range(0..=min_amount);
                builder = builder.start_output(&signer).with_value(Some(decoy.clone()), new_amount as u64).finish_output();
            }
            total += new_amount as u64;
        }
        for _ in 0..their_utxo_count {
            let new_amount = rng.gen_range(0..=min_amount);
            builder = builder.start_output(&recipient).with_value(ADA, new_amount as u64).finish_output();
            for decoy in &decoys {
                let new_amount = rng.gen_range(0..=min_amount);
                builder = builder.start_output(&recipient).with_value(Some(decoy.clone()), new_amount as u64).finish_output();
            }
        }
        (signer, recipient, min_amount as u64, builder.build(), decoys)
    }
}

proptest! {
    #[test]
    fn prop_can_transfer_funds_if_enough_balance(
        (signer, recipient, amount, backend, decoys) in arb_backend_with_enough(),
    ) {
        let u_tx = UnBuiltTransaction::default().with_transfer(amount, recipient.clone(), ADA);

        let my_bal_before = backend.txo_record.balance_at_address(&signer, &ADA);
        let their_bal_before = backend.txo_record.balance_at_address(&recipient, &ADA);
        let mut my_before_decoys = HashMap::new();
        let mut their_before_decoys = HashMap::new();
        for decoy in &decoys {
            let policy = Some(decoy.clone());
            let my_bal_before = backend.txo_record.balance_at_address(&signer, &policy);
            my_before_decoys.insert(policy.clone(), my_bal_before);
            let their_bal_before = backend.txo_record.balance_at_address(&recipient, &policy);
            their_before_decoys.insert(policy.clone(), their_bal_before);
        }
        backend.process(u_tx).unwrap();

        // Check that only the expected ADA moved, and everything else stayed the same.
        let expected = their_bal_before + amount;
        let actual = backend.txo_record.balance_at_address(&recipient, &ADA);
        assert_eq!(expected, actual);
        let expected = my_bal_before - amount;
        let actual = backend.txo_record.balance_at_address(&signer, &ADA);
        assert_eq!(expected, actual);
        for decoy in &decoys {
            let policy = Some(decoy.clone());
            let my_bal_after = backend.txo_record.balance_at_address(&signer, &policy);
            let my_bal_before = my_before_decoys.get(&policy).unwrap();
            assert_eq!(my_bal_before, &my_bal_after);
            let their_bal_after = backend.txo_record.balance_at_address(&recipient, &policy);
            let their_bal_before = their_before_decoys.get(&policy).unwrap();
            assert_eq!(their_bal_before, &their_bal_after);
        }
    }
}

// So IDE recognizes there are tests here
#[test]
fn empty() {}
