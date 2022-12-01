use super::*;
use naumachia::address::Address;
use naumachia::scripts::{MintingPolicy, TxContext};

// TODO: Include real testing!
#[ignore]
#[tokio::test]
async fn mint_arb_tokens() {
    let script = get_policy().unwrap();
    let redeemer = ();
    let ctx = TxContext {
        signer: Address::Raw("hello".to_string()),
    };
    script.execute(redeemer, ctx).unwrap();
}
