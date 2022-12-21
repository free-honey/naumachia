use crate::logic::script::get_policy;
use async_trait::async_trait;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{SCLogic, SCLogicError, SCLogicResult},
    transaction::TxActions,
};

pub mod script;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FreeMintingLogic;

pub enum FreeMintingEndpoints {
    Mint { amount: u64 },
}

#[async_trait]
impl SCLogic for FreeMintingLogic {
    type Endpoints = FreeMintingEndpoints;
    type Lookups = ();
    type LookupResponses = ();
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            FreeMintingEndpoints::Mint { amount } => {
                let recipient = ledger_client
                    .signer()
                    .await
                    .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
                let inner_policy = get_policy().map_err(SCLogicError::PolicyScript)?;
                let policy = Box::new(inner_policy);
                let actions = TxActions::v1().with_mint(
                    amount,
                    Some("FREEEEEE".to_string()),
                    &recipient,
                    (),
                    policy,
                );
                Ok(actions)
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        _query: Self::Lookups,
        _ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        unimplemented!()
    }
}
