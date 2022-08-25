use super::*;
use crate::ledger_client::in_memory_ledger::{InMemoryLedgerClient, TestBackendsBuilder};

prop_compose! {
    fn arb_backend_with_enough()(
        min_amount: u32,
        mut rng in arb_rng(),
        their_utxo_count: u8,
        decoys in prop::collection::vec(arb_policy_id(), 0..10),
    ) -> (FakeAddress, FakeAddress, u64, Backend<(),(), InMemoryLedgerClient<(),()>>, Vec<PolicyId>) {
        let signer = FakeAddress::new("alice");
        let recipient = FakeAddress::new("bob");
        let mut total: u64 = 0;
        let mut builder = TestBackendsBuilder::<(), ()>::new(&signer);
        while total < min_amount as u64 {
            let new_amount = rng.gen_range(0..=min_amount);
            builder = builder.start_output(&signer).with_value(PolicyId::ADA, new_amount as u64).finish_output();
            for decoy in &decoys {
                let new_amount = rng.gen_range(0..=min_amount);
                builder = builder.start_output(&signer).with_value(decoy.to_owned(), new_amount as u64).finish_output();
            }
            total += new_amount as u64;
        }
        for _ in 0..their_utxo_count {
            let new_amount = rng.gen_range(0..=min_amount);
            builder = builder.start_output(&recipient).with_value(PolicyId::ADA, new_amount as u64).finish_output();
            for decoy in &decoys {
                let new_amount = rng.gen_range(0..=min_amount);
                builder = builder.start_output(&recipient).with_value(decoy.to_owned(), new_amount as u64).finish_output();
            }
        }
        (signer, recipient, min_amount as u64, builder.build(), decoys)
    }
}

// TODO: Cleanup. Since it's a prop test, prolly don't want to break it out into multiple tests,
//   but it's really hard to read
proptest! {
    #![proptest_config(ProptestConfig {
    cases: 10, .. ProptestConfig::default()
    })]
    #[test]
    fn prop_can_transfer_funds_if_enough_balance(
        (signer, recipient, amount, backend, decoys) in arb_backend_with_enough(),
    ) {
        let u_tx = UnBuiltTransaction::default().with_transfer(amount, recipient.clone(), PolicyId::ADA);

        let my_bal_before = backend.txo_record.balance_at_address(&signer, &PolicyId::ADA);
        let their_bal_before = backend.txo_record.balance_at_address(&recipient, &PolicyId::ADA);
        let mut my_before_decoys = HashMap::new();
        let mut their_before_decoys = HashMap::new();
        for policy_id in &decoys {
            let my_bal_before = backend.txo_record.balance_at_address(&signer, &policy_id);
            my_before_decoys.insert(policy_id.clone(), my_bal_before);
            let their_bal_before = backend.txo_record.balance_at_address(&recipient, &policy_id);
            their_before_decoys.insert(policy_id.clone(), their_bal_before);
        }
        backend.process(u_tx).unwrap();

        // Check that only the expected ADA moved, and everything else stayed the same.
        let expected = their_bal_before + amount;
        let actual = backend.txo_record.balance_at_address(&recipient, &PolicyId::ADA);
        assert_eq!(expected, actual);
        let expected = my_bal_before - amount;
        let actual = backend.txo_record.balance_at_address(&signer, &PolicyId::ADA);
        assert_eq!(expected, actual);
        for policy_id in &decoys {
            let my_bal_after = backend.txo_record.balance_at_address(&signer, &policy_id);
            let my_bal_before = my_before_decoys.get(&policy_id).unwrap();
            assert_eq!(my_bal_before, &my_bal_after);
            let their_bal_after = backend.txo_record.balance_at_address(&recipient, &policy_id);
            let their_bal_before = their_before_decoys.get(&policy_id).unwrap();
            assert_eq!(their_bal_before, &their_bal_after);
        }
    }
}
