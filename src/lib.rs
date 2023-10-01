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
pub mod output;
/// `PolicyId` type module
pub mod policy_id;
pub mod scripts;
pub mod smart_contract;
pub mod transaction;

pub mod backend;
pub mod trireme_ledger_client;
pub mod values;
