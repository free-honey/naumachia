use crate::lookups::get_my_accounts;
use crate::{
    scripts::{
        checking_account_validtor::checking_account_validator, pull_validator::pull_validator,
        spend_token_policy::spend_token_policy,
    },
    withdraw::withdraw_from_account,
};
use async_trait::async_trait;
use datum::{AllowedPuller, CheckingAccount, CheckingAccountDatums};
use endpoints::{
    add_puller::add_puller, fund_account::fund_account, init_account::init_account,
    pull::pull_from_account, remove_puller::remove_puller,
};
use naumachia::logic::error::SCLogicResult;
use naumachia::{
    ledger_client::LedgerClient, logic::SCLogic, output::OutputId, scripts::context::PubKeyHash,
    transaction::TxActions, Address,
};
use thiserror::Error;

pub mod datum;
mod endpoints;
mod lookups;
pub mod scripts;
mod withdraw;

pub const CHECKING_ACCOUNT_NFT_ASSET_NAME: &str = "CHECKING ACCOUNT";
pub const SPEND_TOKEN_ASSET_NAME: &str = "SPEND TOKEN";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TimeLockedLogic;

#[derive(Debug)]
pub enum CheckingAccountEndpoints {
    // Owner Endpoints
    /// Create a new checking account
    InitAccount { starting_lovelace: u64 },
    /// Allow puller to pull amount from checking account every period,
    /// starting on the next_pull time, in milliseconds POSIX
    AddPuller {
        checking_account_nft: String,
        puller: PubKeyHash,
        amount_lovelace: u64,
        period: i64,
        next_pull: i64,
    },
    /// Disallow puller from accessing account account
    RemovePuller { output_id: OutputId },
    /// Add funds to checking account
    FundAccount {
        output_id: OutputId,
        fund_amount: u64,
    },
    /// Remove funds from a checking account
    WithdrawFromAccount {
        output_id: OutputId,
        withdraw_amount: u64,
    },
    // Puller Endpoints
    /// Use allowed puller validator to pull from checking account
    PullFromCheckingAccount {
        allow_pull_output_id: OutputId,
        checking_account_output_id: OutputId,
        amount: u64,
    },
}

#[derive(Debug)]
pub enum CheckingAccountLookups {
    MyAccounts,
}

#[derive(Debug)]
pub enum CheckingAccountLookupResponses {
    MyAccounts(Vec<Account>),
}

#[derive(Debug)]
pub struct Account {
    pub balance_ada: f64,
    pub nft: Option<String>,
    pub pullers: Vec<AccountPuller>,
}

#[derive(Debug)]
pub struct AccountPuller {
    pub puller: PubKeyHash,
    pub amount_lovelace: u64,
    pub period: i64,
    pub next_pull: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CheckingAccountLogic;

#[derive(Debug, Error)]
pub enum CheckingAccountError {
    #[error("Could not find a valid input UTxO")]
    InputNotFound,
    #[error("Could not find an output with id: {0:?}")]
    OutputNotFound(OutputId),
    #[error("Expected datum on output with id: {0:?}")]
    DatumNotFoundForOutput(OutputId),
    #[error("You are trying to withdraw more than is in account")]
    CannotWithdrawSpecifiedAmount,
    #[error("Address isn't valid: {0:?}")]
    InvalidAddress(Address),
    #[error("Too early to pull: next_pull: {next_pull}, current_time: {current_time}")]
    TooEarlyToPull { next_pull: i64, current_time: i64 },
}

#[async_trait]
impl SCLogic for CheckingAccountLogic {
    type Endpoints = CheckingAccountEndpoints;
    type Lookups = CheckingAccountLookups;
    type LookupResponses = CheckingAccountLookupResponses;
    type Datums = CheckingAccountDatums;
    type Redeemers = ();

    async fn handle_endpoint<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &Record,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            CheckingAccountEndpoints::InitAccount { starting_lovelace } => {
                init_account(ledger_client, starting_lovelace).await
            }
            CheckingAccountEndpoints::AddPuller {
                checking_account_nft,
                puller,
                amount_lovelace,
                period,
                next_pull,
            } => {
                add_puller(
                    ledger_client,
                    checking_account_nft,
                    puller,
                    amount_lovelace,
                    period,
                    next_pull,
                )
                .await
            }
            CheckingAccountEndpoints::RemovePuller { output_id } => {
                remove_puller(ledger_client, output_id).await
            }
            CheckingAccountEndpoints::FundAccount {
                output_id,
                fund_amount,
            } => fund_account(ledger_client, output_id, fund_amount).await,
            CheckingAccountEndpoints::WithdrawFromAccount {
                output_id,
                withdraw_amount,
            } => withdraw_from_account(ledger_client, output_id, withdraw_amount).await,
            CheckingAccountEndpoints::PullFromCheckingAccount {
                allow_pull_output_id,
                checking_account_output_id,
                amount,
            } => {
                pull_from_account(
                    ledger_client,
                    allow_pull_output_id,
                    checking_account_output_id,
                    amount,
                )
                .await
            }
        }
    }

    async fn lookup<Record: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &Record,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            CheckingAccountLookups::MyAccounts => get_my_accounts(ledger_client).await,
        }
    }
}
