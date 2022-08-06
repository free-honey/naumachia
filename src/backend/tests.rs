use super::*;
use crate::address::ADA;
use crate::backend::fake_backend::TestBackendsBuilder;

#[test]
fn can_transfer_funds_if_enough_balance() {
    let amount = 10;
    let signer = Address::new("me");
    let recipient = Address::new("them");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer)
        .start_output(&signer)
        .with_value(ADA, 20)
        .finish_output()
        .build();
    let u_tx = UnBuiltTransaction::default().with_transfer(amount, recipient.clone(), ADA);

    backend.process(u_tx).unwrap();

    let expected = amount;
    let actual = backend.txo_record.balance_at_address(&recipient, &ADA);
    assert_eq!(expected, actual);
}
