use crate::{
    checking_account_validator, pull_validator, spend_token_policy, AllowedPuller,
    CheckingAccountDatums, CheckingAccountError, SPEND_TOKEN_ASSET_NAME,
};
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogicError, SCLogicResult},
    scripts::context::{pub_key_hash_from_address_if_available, PubKeyHash},
    scripts::MintingPolicy,
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};

pub async fn add_puller<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    checking_account_nft_id: String,
    puller: PubKeyHash,
    amount_lovelace: u64,
    period: i64,
    next_pull: i64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let me = ledger_client
        .signer_base_address()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let owner = pub_key_hash_from_address_if_available(&me).ok_or(SCLogicError::Endpoint(
        Box::new(CheckingAccountError::InvalidAddress(me.clone())),
    ))?;

    let nft_id_bytes = hex::decode(checking_account_nft_id).unwrap();
    let my_pubkey = pub_key_hash_from_address_if_available(&me).unwrap();

    let parameterized_spending_token_policy = spend_token_policy().unwrap();
    let policy = parameterized_spending_token_policy
        .apply(nft_id_bytes.clone().into())
        .unwrap()
        .apply(my_pubkey.into())
        .unwrap();

    let id = policy.id().unwrap();
    let boxed_policy = Box::new(policy);

    let address = pull_validator()
        .map_err(SCLogicError::ValidatorScript)?
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let mut values = Values::default();
    values.add_one_value(
        &PolicyId::NativeToken(id.clone(), Some(SPEND_TOKEN_ASSET_NAME.to_string())),
        1,
    );
    let datum = AllowedPuller {
        owner,
        puller,
        amount_lovelace,
        next_pull,
        period,
        spending_token: hex::decode(&id).unwrap(), // TODO
        checking_account_nft: nft_id_bytes,
    }
    .into();
    let actions = TxActions::v2()
        .with_mint(
            1,
            Some(SPEND_TOKEN_ASSET_NAME.to_string()),
            (),
            boxed_policy,
        )
        .with_script_init(datum, values, address);
    Ok(actions)
}
