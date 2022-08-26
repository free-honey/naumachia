use super::*;
use crate::{
    error::Error,
    ledger_client::in_memory_ledger::TestBackendsBuilder,
    scripts::TxContext,
    scripts::{MintingPolicy, ScriptError, ScriptResult},
};

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

    fn id(&self) -> PolicyId {
        PolicyId::native_token("OnlyAliceCanMint")
    }
}

#[tokio::test]
async fn mint__alice_can_mint() {
    let signer = Address::new("alice");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<(), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    backend.process(u_tx).await.unwrap();

    let policy_id = AliceCanMintPolicy.id();

    let expected = 100;
    let actual = backend
        .txo_record
        .balance_at_address(&signer, &policy_id)
        .await;

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn mint__bob_cannot_mint() {
    let signer = Address::new("bob");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<(), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    let actual_err = backend.process(u_tx).await.unwrap_err();

    let matches = matches!(actual_err, Error::Script(ScriptError::FailedToExecute(_)),);
    assert!(matches);
}
