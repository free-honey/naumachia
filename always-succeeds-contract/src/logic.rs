use crate::logic::script::AlwaysSucceedsScript;
use async_trait::async_trait;
use naumachia::ledger_client::cml_client::key_manager::TESTNET;
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::SCLogicError,
    logic::{SCLogic, SCLogicResult},
    output::OutputId,
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};

pub mod script;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlwaysSucceedsLogic;

pub enum AlwaysSucceedsEndpoints {
    Lock { amount: u64 },
    Claim { output: OutputId },
}

pub enum AlwaysSucceedsLookups {
    ListActiveContracts { count: u64 },
}

#[async_trait]
impl SCLogic for AlwaysSucceedsLogic {
    type Endpoint = AlwaysSucceedsEndpoints;
    type Lookup = ();
    type LookupResponse = ();
    type Datum = ();
    type Redeemer = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        endpoint: Self::Endpoint,
        _ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
        match endpoint {
            AlwaysSucceedsEndpoints::Lock { amount } => Self::impl_lock(amount),
            AlwaysSucceedsEndpoints::Claim { output } => Self::impl_claim(output),
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datum, Self::Redeemer>>(
        _endpoint: Self::Lookup,
        _ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponse> {
        todo!()
    }
}

impl AlwaysSucceedsLogic {
    // fn impl_lock(amount: u64) -> SCLogicResult<TxActions<Self::Datum, Self::Redeemer>> {
    fn impl_lock(amount: u64) -> SCLogicResult<TxActions<(), ()>> {
        let mut values = Values::default();
        values.add_one_value(&PolicyId::ADA, amount);
        let script = AlwaysSucceedsScript::try_new().map_err(SCLogicError::ValidatorScript)?;
        // TODO: Need to pass through network param
        let address = script
            .address(TESTNET)
            .map_err(SCLogicError::ValidatorScript)?;
        let tx_actions = TxActions::default().with_script_init((), values, address);
        Ok(tx_actions)
    }

    fn impl_claim(_output: OutputId) -> SCLogicResult<TxActions<(), ()>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
