use super::*;
use crate::backend::in_memory_record::TestBackendsBuilder;
use crate::scripts::{MintingPolicy, ScriptError, ScriptResult};

struct AliceCanMintPolicy;

impl MintingPolicy for AliceCanMintPolicy {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()> {
        if ctx.signer == Address::new("alice") {
            Ok(())
        } else {
            Err(ScriptError::FailedToExecute(
                "Signer must be `alice`".to_string(),
            ))
        }
    }

    fn address(&self) -> Address {
        Address::new("OnlyAliceCanMint")
    }
}

#[test]
fn mint__alice_can_mint() {
    let signer = Address::new("alice");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<(), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    backend.process(u_tx).unwrap();

    let policy_id = Some(AliceCanMintPolicy.address());

    let expected = 100;
    let actual = backend.txo_record.balance_at_address(&signer, &policy_id);

    assert_eq!(expected, actual);
}

#[test]
fn mint__bob_cannot_mint() {
    let signer = Address::new("bob");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<(), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    let actual_err = backend.process(u_tx).unwrap_err();
    let expected_err = "Signer must be `alice`".to_string();

    assert_eq!(expected_err, actual_err);
}
