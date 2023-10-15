#![warn(missing_docs)]

//! Naumachia Offchain Smart Contract Framework!

use crate::{
    policy_id::PolicyId,
    transaction::{TxActions, UnbuiltTransaction},
};

pub use pallas_addresses::{Address, Network};

pub mod error;

/// Ledger client module
pub mod ledger_client;
/// Smart contract logic module
pub mod logic;
/// UTxO data types module
pub mod output;
/// `PolicyId` type module
pub mod policy_id;
/// On-chain script module
pub mod scripts;
/// Smart contract module
pub mod smart_contract;
/// Transaction module
pub mod transaction;

/// Types and helpers for working with the Trireme CLI
pub mod trireme_ledger_client;
pub mod values;
