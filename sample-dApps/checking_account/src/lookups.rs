use crate::{
    checking_account_validator, Account, CheckingAccountDatums, CheckingAccountLookupResponses,
    CHECKING_ACCOUNT_NFT_ASSET_NAME,
};
use naumachia::address::PolicyId;
use naumachia::ledger_client::LedgerClient;
use naumachia::logic::{SCLogicError, SCLogicResult};
use naumachia::scripts::context::pub_key_hash_from_address_if_available;
use naumachia::scripts::ValidatorCode;

pub async fn get_my_accounts<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
) -> SCLogicResult<CheckingAccountLookupResponses> {
    let my_address = ledger_client
        .signer_base_address()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let my_pubkey_hash = pub_key_hash_from_address_if_available(&my_address).unwrap();
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let checking_account_address = validator
        .address(0)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let outputs = ledger_client
        .all_outputs_at_address(&checking_account_address)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let my_accounts = outputs
        .into_iter()
        .filter(|output| {
            if let Some(datum) = output.typed_datum() {
                if let CheckingAccountDatums::CheckingAccount(inner) = datum {
                    inner.owner == my_pubkey_hash
                } else {
                    false
                }
            } else {
                false
            }
        })
        .map(|output| {
            let values = output.values();
            let balance_ada = values
                .get(&PolicyId::Lovelace)
                .map(|lovelace| (lovelace as f64) / 1_000_000.0) // TODO: fallible
                .unwrap_or_default();
            let nft = values
                .as_iter()
                .find(|(policy_id, amt)| {
                    if let Some(asset_name) = policy_id.asset_name() {
                        amt == &&1 && asset_name == CHECKING_ACCOUNT_NFT_ASSET_NAME
                    // This is kinda fragile
                    } else {
                        false
                    }
                })
                .map(|value| value.0.id());
            Account { balance_ada, nft }
        })
        .collect();
    Ok(CheckingAccountLookupResponses::MyAccounts(my_accounts))
}