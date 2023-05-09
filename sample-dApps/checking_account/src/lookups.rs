use crate::{
    checking_account_validator, pull_validator, Account, AccountPuller, CheckingAccountDatums,
    CheckingAccountLookupResponses, CHECKING_ACCOUNT_NFT_ASSET_NAME,
};
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogicError, SCLogicResult},
    scripts::context::pub_key_hash_from_address_if_available,
    scripts::ValidatorCode,
};

pub async fn get_my_accounts<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
) -> SCLogicResult<CheckingAccountLookupResponses> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let my_address = ledger_client
        .signer_base_address()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let my_pubkey_hash = pub_key_hash_from_address_if_available(&my_address).unwrap();
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let checking_account_address = validator
        .address(network)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let outputs = ledger_client
        .all_outputs_at_address(&checking_account_address)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let balance_and_nft_iter = outputs
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

            (balance_ada, nft)
        });

    let mut accounts = Vec::new();
    for (balance, maybe_nft) in balance_and_nft_iter {
        let pullers = if let Some(nft) = &maybe_nft {
            find_pullers_for_nft(nft, ledger_client).await?
        } else {
            Vec::new()
        };
        accounts.push(Account {
            balance_ada: balance,
            nft: maybe_nft,
            pullers,
        });
    }
    Ok(CheckingAccountLookupResponses::MyAccounts(accounts))
}

async fn find_pullers_for_nft<LC: LedgerClient<CheckingAccountDatums, ()>>(
    nft_policy_id: &str,
    ledger_client: &LC,
) -> SCLogicResult<Vec<AccountPuller>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let address = pull_validator()
        .map_err(SCLogicError::ValidatorScript)?
        .address(network)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let outputs = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let pullers = outputs
        .into_iter()
        .filter_map(|output| {
            let datum = output.typed_datum().unwrap(); // TODO
            if let CheckingAccountDatums::AllowedPuller(inner) = datum {
                let nft_bytes = hex::decode(nft_policy_id).unwrap(); // TODO
                if inner.checking_account_nft == nft_bytes {
                    let account_puller = AccountPuller {
                        puller: inner.puller,
                        amount_lovelace: inner.amount_lovelace,
                        period: inner.period,
                        next_pull: inner.next_pull,
                    };
                    Some(account_puller)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    Ok(pullers)
}
