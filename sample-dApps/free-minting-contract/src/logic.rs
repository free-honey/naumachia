use crate::logic::script::get_policy;
use async_trait::async_trait;
use naumachia::{
    ledger_client::LedgerClient,
    logic::{
        error::{
            SCLogicError,
            SCLogicResult,
        },
        SCLogic,
    },
    transaction::TxActions,
};

pub mod script;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FreeMintingLogic;

#[derive(Debug)]
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
        _ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            FreeMintingEndpoints::Mint { amount } => {
                let inner_policy = get_policy().map_err(SCLogicError::PolicyScript)?;
                let policy = Box::new(inner_policy);
                let actions = TxActions::v1().with_mint(
                    amount,
                    Some("FREEEEEE".to_string()),
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
