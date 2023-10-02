use crate::{pull_validator, CheckingAccountDatums, CheckingAccountError};
use naumachia::logic::error::{SCLogicError, SCLogicResult};
use naumachia::{
    ledger_client::LedgerClient, output::OutputId, scripts::Validator, transaction::TxActions,
};

pub async fn remove_puller<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let validator = pull_validator().map_err(SCLogicError::ValidatorScript)?;
    let address = validator
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let redeemer = ();
    let script = Box::new(validator);
    let actions = TxActions::v2().with_script_redeem(output, redeemer, script);
    Ok(actions)
}
