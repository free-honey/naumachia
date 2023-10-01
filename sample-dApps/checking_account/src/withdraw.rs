use crate::{checking_account_validator, CheckingAccountDatums, CheckingAccountError};
use naumachia::logic::error::{SCLogicError, SCLogicResult};
use naumachia::{
    ledger_client::LedgerClient, output::OutputId, policy_id::PolicyId, scripts::ValidatorCode,
    transaction::TxActions, values::Values,
};

pub async fn withdraw_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
    amount: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let address = validator
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let new_datum = output
        .typed_datum()
        .ok_or(CheckingAccountError::DatumNotFoundForOutput(
            output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let redeemer = ();
    let script = Box::new(validator);
    let old_values = output.values().to_owned();
    let mut sub_values = Values::default();
    sub_values.add_one_value(&PolicyId::Lovelace, amount);
    let new_value = old_values
        .try_subtract(&sub_values)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .ok_or(CheckingAccountError::OutputNotFound(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script)
        .with_script_init(new_datum, new_value, address);
    Ok(actions)
}
