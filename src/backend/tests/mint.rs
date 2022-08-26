use super::*;
use crate::{
    error::Error,
    ledger_client::in_memory_ledger::TestBackendsBuilder,
    scripts::TxContext,
    scripts::{MintingPolicy, ScriptError, ScriptResult},
};

struct AliceCanMintPolicy;

impl<Address: ValidAddress> MintingPolicy<Address> for AliceCanMintPolicy {
    fn execute(&self, ctx: TxContext<Address>) -> ScriptResult<()> {
        if ctx.signer == "alice".to_string().into() {
            Ok(())
        } else {
            Err(ScriptError::FailedToExecute(
                "Signer must be `alice`".to_string(),
            ))
        }
    }

    fn id(&self) -> PolicyId {
        PolicyId::native_token("OnlyAliceCanMint")
    }
}

#[test]
fn mint__alice_can_mint() {
    let signer = FakeAddress::new("alice");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<FakeAddress, (), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    backend.process(u_tx).unwrap();

    let policy_id = <AliceCanMintPolicy as MintingPolicy<.id();

    let expected = 100;
    let actual = backend
        .ledger_client
        .balance_at_address(&signer, &policy_id);

    assert_eq!(expected, actual);
}

#[test]
fn mint__bob_cannot_mint() {
    let signer = FakeAddress::new("bob");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<FakeAddress, (), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    let actual_err = backend.process(u_tx).unwrap_err();

    let matches = matches!(actual_err, Error::Script(ScriptError::FailedToExecute(_)),);
    assert!(matches);
}
