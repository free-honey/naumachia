use super::*;
use crate::ledger_client::LedgerClientError;
use crate::scripts::context::{pub_key_hash_from_address_if_available, TxContext};
use crate::scripts::ExecutionCost;
use crate::{
    ledger_client::test_ledger_client::TestLedgerClientBuilder,
    scripts::{MintingPolicy, ScriptError, ScriptResult},
};

struct AliceCanMintPolicy;

const ALICE: &str = "addr_test1qrmezjhpelwzvz83wjl0e6mx766de7j3nksu2338s00yzx870xyxfa97xyz2zn5rknyntu5g0c66s7ktjnx0p6f0an6s3dyxwr";
const BOB: &str = "addr_test1qpuy2q9xel76qxdw8r29skldzc876cdgg9cugfg7mwh0zvpg3292mxuf3kq7nysjumlxjrlsfn9tp85r0l54l29x3qcs7nvyfm";

impl<R> MintingPolicy<R> for AliceCanMintPolicy {
    fn execute(&self, _redeemer: R, ctx: TxContext) -> ScriptResult<ExecutionCost> {
        let alice_address = Address::from_bech32(ALICE).unwrap();
        let alice_pubkey_hash = pub_key_hash_from_address_if_available(&alice_address).unwrap();
        if ctx.signer == alice_pubkey_hash {
            Ok(ExecutionCost::default())
        } else {
            Err(ScriptError::FailedToExecute(
                "Signer must be `alice`".to_string(),
            ))
        }
    }

    fn id(&self) -> ScriptResult<String> {
        Ok(hex::encode(vec![1, 2, 3, 4, 5]))
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}

#[tokio::test]
async fn mint__alice_can_mint() {
    let signer = Address::from_bech32(ALICE).unwrap();
    let ledger_client = TestLedgerClientBuilder::<(), ()>::new(&signer).build_in_memory();
    let amount = 100;

    let asset_name = None;
    let actions: TxActions<(), ()> =
        TxActions::v1().with_mint(amount, asset_name.clone(), (), Box::new(AliceCanMintPolicy));

    let u_tx = actions.to_unbuilt_tx().unwrap();
    ledger_client.issue(u_tx).await.unwrap();

    let id = <AliceCanMintPolicy as MintingPolicy<()>>::id(&AliceCanMintPolicy).unwrap();
    let policy_id = PolicyId::native_token(&id, &asset_name);

    let expected = 100;
    let actual = ledger_client
        .balance_at_address(&signer, &policy_id)
        .await
        .unwrap();

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn mint__bob_cannot_mint() {
    let signer = Address::from_bech32(BOB).unwrap();
    let ledger_client = TestLedgerClientBuilder::<(), ()>::new(&signer).build_in_memory();
    let amount = 100;

    let asset_name = None;
    let actions: TxActions<(), ()> =
        TxActions::v1().with_mint(amount, asset_name, (), Box::new(AliceCanMintPolicy));

    let u_tx = actions.to_unbuilt_tx().unwrap();
    let actual_err = ledger_client.issue(u_tx).await.unwrap_err();

    let matches = matches!(actual_err, LedgerClientError::FailedToIssueTx(_));
    assert!(matches);
}
