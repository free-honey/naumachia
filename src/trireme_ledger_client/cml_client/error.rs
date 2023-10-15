use crate::ledger_client::LedgerClientError;
use pallas_addresses::Address;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum CMLLCError {
    #[error("CML JsError: {0:?}")]
    JsError(String),
    #[error("Address Error: {0:?}")]
    Address(#[from] pallas_addresses::Error),
    #[error("Scrolls Client: {0:?}")]
    ScrollsClient(#[from] scrolls_client::error::Error),
    #[error("Ogmios Client: {0:?}")]
    OgmiosClient(#[from] ogmios_client::Error),
    #[error("Ogmios Response: {0:?}")]
    OgmiosResponse(String),
    #[error("Not a valid BaseAddress")]
    InvalidBaseAddr,
    #[error("Error from ledger implementation: {0:?}")]
    LedgerError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error in key manager implementation: {0:?}")]
    KeyError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Unbuilt output does not have sufficient ADA")]
    InsufficientADA,
    #[error("Error while deserializing: {0:?}")]
    Deserialize(String),
    #[error("Failed to parse Hex")]
    // Hex(Box<dyn std::error::Error + Send + Sync>),
    Hex(#[from] hex::FromHexError),
    #[error("Invalid Policy Id: {0:?}")]
    InvalidPolicyId(String),
}

/// Convenience function for wrapping a `CMLLCError` in a [`LedgerClientError`] `FailedToRetrieveOutputsAt` variant
pub fn as_failed_to_retrieve_by_address(
    addr: &Address,
) -> impl Fn(CMLLCError) -> LedgerClientError + '_ {
    move |e| LedgerClientError::FailedToRetrieveOutputsAt(addr.to_owned(), Box::new(e))
}

/// Convenience function for wrapping a `CMLLCError` in a [`LedgerClientError`] `FailedToRetrieveOutputsAt` variant
pub fn as_failed_to_issue_tx<E: std::error::Error + Send + Sync + 'static>(
    error: E,
) -> LedgerClientError {
    LedgerClientError::FailedToIssueTx(Box::new(error))
}

/// Convenience function for wrapping a `CMLLCError` in a [`LedgerClientError`] `FailedToGetBlockTime` variant
pub fn as_failed_to_get_block_time<E: std::error::Error + Send + Sync + 'static>(
    error: E,
) -> LedgerClientError {
    LedgerClientError::FailedToGetBlockTime(Box::new(error))
}

#[allow(missing_docs)]
pub type Result<T, E = CMLLCError> = std::result::Result<T, E>;
