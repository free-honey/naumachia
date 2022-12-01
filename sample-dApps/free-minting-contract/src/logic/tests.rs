use super::*;
use naumachia::address::Address;
use naumachia::ledger_client::test_ledger_client::TestBackendsBuilder;
use naumachia::scripts::{MintingPolicy, TxContext};
use naumachia::smart_contract::{SmartContract, SmartContractTrait};

#[tokio::test]
async fn mint_arb_tokens() {
    let script = get_policy().unwrap();
    let redeemer = ();
    let ctx = TxContext {
        signer: Address::Raw("hello".to_string()),
    };
    script.execute(redeemer, ctx);
}
