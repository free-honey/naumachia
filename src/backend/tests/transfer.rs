use super::*;
use crate::{
    ledger_client::test_ledger_client::{
        in_memory_storage::InMemoryStorage, TestBackendsBuilder, TestLedgerClient,
    },
    PolicyId,
};
use std::collections::HashMap;
use tokio::runtime::Runtime;

prop_compose! {
    fn arb_backend_with_enough()(
        min_amount: u32,
        mut rng in arb_rng(),
        their_utxo_count: u8,
        decoys in prop::collection::vec(arb_policy_id(), 0..10),
    ) -> (Address, Address, u64, Backend<(),(), TestLedgerClient<(),(), InMemoryStorage<()>>>, Vec<PolicyId>) {
        let signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let recipient = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();
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
        (signer, recipient, min_amount as u64, builder.build_in_memory(), decoys)
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
        let rt = Runtime::new().unwrap();

        rt.block_on(inner_test(signer, recipient, amount, backend, decoys))
    }
}

async fn inner_test(
    signer: Address,
    recipient: Address,
    amount: u64,
    backend: Backend<(), (), TestLedgerClient<(), (), InMemoryStorage<()>>>,
    decoys: Vec<PolicyId>,
) {
    let u_tx = TxActions::v1().with_transfer(amount, recipient.clone(), PolicyId::ADA);

    let my_bal_before = backend
        .ledger_client
        .balance_at_address(&signer, &PolicyId::ADA)
        .await
        .unwrap();
    let their_bal_before = backend
        .ledger_client
        .balance_at_address(&recipient, &PolicyId::ADA)
        .await
        .unwrap();
    let mut my_before_decoys = HashMap::new();
    let mut their_before_decoys = HashMap::new();
    for policy_id in &decoys {
        let my_bal_before = backend
            .ledger_client
            .balance_at_address(&signer, policy_id)
            .await
            .unwrap();
        my_before_decoys.insert(policy_id.clone(), my_bal_before);
        let their_bal_before = backend
            .ledger_client
            .balance_at_address(&recipient, policy_id)
            .await
            .unwrap();
        their_before_decoys.insert(policy_id.clone(), their_bal_before);
    }
    backend.process(u_tx).await.unwrap();

    // Check that only the expected ADA moved, and everything else stayed the same.
    let expected = their_bal_before + amount;
    let actual = backend
        .ledger_client
        .balance_at_address(&recipient, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(expected, actual);
    let expected = my_bal_before - amount;
    let actual = backend
        .ledger_client
        .balance_at_address(&signer, &PolicyId::ADA)
        .await
        .unwrap();
    assert_eq!(expected, actual);
    for policy_id in &decoys {
        let my_bal_after = backend
            .ledger_client
            .balance_at_address(&signer, policy_id)
            .await
            .unwrap();
        let my_bal_before = my_before_decoys.get(policy_id).unwrap();
        assert_eq!(my_bal_before, &my_bal_after);
        let their_bal_after = backend
            .ledger_client
            .balance_at_address(&recipient, policy_id)
            .await
            .unwrap();
        let their_bal_before = their_before_decoys.get(policy_id).unwrap();
        assert_eq!(their_bal_before, &their_bal_after);
    }
}
