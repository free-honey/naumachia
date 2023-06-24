use crate::{
    checking_account_validator, spend_token_policy, CheckingAccount, CheckingAccountDatums,
    CheckingAccountError, CHECKING_ACCOUNT_NFT_ASSET_NAME,
};
use nau_scripts::{one_shot, one_shot::OutputReference};
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogicError, SCLogicResult},
    output::Output,
    scripts::context::pub_key_hash_from_address_if_available,
    scripts::ValidatorCode,
    scripts::{MintingPolicy, ScriptError},
    transaction::TxActions,
    values::Values,
};

pub async fn init_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    starting_lovelace: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let owner = ledger_client
        .signer_base_address()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let my_input = select_any_above_min(ledger_client).await?;
    let nft = one_shot::get_parameterized_script().map_err(SCLogicError::PolicyScript)?;
    let nft_policy = nft
        .apply(OutputReference::from(&my_input))
        .map_err(|e| ScriptError::FailedToConstruct(format!("{:?}", e)))
        .map_err(SCLogicError::PolicyScript)?;
    let spending_token_policy_parameterized =
        spend_token_policy().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let nft_script_id = nft_policy.id().unwrap();
    let nft_script_id_bytes = hex::decode(nft_script_id.clone()).unwrap();
    let owner_pubkey = pub_key_hash_from_address_if_available(&owner).unwrap();
    let spending_token_policy = spending_token_policy_parameterized
        .apply(nft_script_id_bytes.into())
        .unwrap()
        .apply(owner_pubkey.clone().into())
        .unwrap();
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let spend_token_policy = spending_token_policy.id().unwrap();
    let address = validator
        .address(network)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let spend_token_id = hex::decode(&spend_token_policy).unwrap();
    let datum = CheckingAccount {
        owner: owner_pubkey,
        spend_token_policy: spend_token_id,
    }
    .into();
    let boxed_nft_policy = Box::new(nft_policy);
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, starting_lovelace);
    values.add_one_value(
        &PolicyId::NativeToken(
            nft_script_id,
            Some(CHECKING_ACCOUNT_NFT_ASSET_NAME.to_string()),
        ),
        1,
    );

    println!("address: {:?}", address.to_bech32());
    let actions = TxActions::v2()
        .with_script_init(datum, values, address)
        .with_specific_input(my_input)
        .with_mint(
            1,
            Some(CHECKING_ACCOUNT_NFT_ASSET_NAME.to_string()),
            (),
            boxed_nft_policy,
        );
    Ok(actions)
}

async fn select_any_above_min<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
) -> SCLogicResult<Output<CheckingAccountDatums>> {
    const MIN_LOVELACE: u64 = 5_000_000;
    let me = ledger_client
        .signer_base_address()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let selected = ledger_client
        .all_outputs_at_address(&me)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .iter()
        .filter_map(|input| {
            if let Some(ada_value) = input.values().get(&PolicyId::Lovelace) {
                if ada_value > MIN_LOVELACE {
                    Some(input)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .pop()
        .ok_or(SCLogicError::Endpoint(Box::new(
            CheckingAccountError::InputNotFound,
        )))?
        .to_owned();
    Ok(selected)
}
