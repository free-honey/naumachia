use super::*;
use crate::backend::in_memory_record::TestBackendsBuilder;
use crate::scripts::MintingPolicy;

struct AliceCanMintPolicy;

impl MintingPolicy for AliceCanMintPolicy {
    fn execute(&self, ctx: TxContext) -> Result<()> {
        todo!()
        // if ctx.signer == Address::new("alice") {
        //    Ok(())
        // } else {
        //     Err("Signer must be `alice`".to_string())
        // }
    }

    fn address(&self) -> Address {
        Address::new("OnlyAliceCanMint")
    }
}

#[test]
fn mint__alice_can_mint() {
    let signer = Address::new("alice");
    let mut backend = TestBackendsBuilder::<(), ()>::new(&signer).build();
    let amount = 100;

    let u_tx: UnBuiltTransaction<(), ()> =
        UnBuiltTransaction::default().with_mint(amount, &signer, Box::new(AliceCanMintPolicy));

    backend.process(u_tx).unwrap();

    let policy_id = Some(AliceCanMintPolicy.address());

    let expected = 100;
    let actual = backend.txo_record.balance_at_address(&signer, &policy_id);

    assert_eq!(expected, actual);
}
