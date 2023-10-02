use crate::{checking_account_validator, CheckingAccountDatums, CheckingAccountError};
use naumachia::logic::error::{SCLogicError, SCLogicResult};
use naumachia::{
    ledger_client::LedgerClient, output::OutputId, policy_id::PolicyId, scripts::Validator,
    transaction::TxActions,
};

pub async fn fund_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    output_id: OutputId,
    amount: u64,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let network = ledger_client
        .network()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let checking_account_validator =
        checking_account_validator().map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    let checking_account_address = checking_account_validator
        .address(network)
        .map_err(SCLogicError::ValidatorScript)?;
    let output = ledger_client
        .all_outputs_at_address(&checking_account_address)
        .await
        .map_err(|e| SCLogicError::Lookup(Box::new(e)))?
        .into_iter()
        .find(|o| o.id() == &output_id)
        .ok_or(CheckingAccountError::OutputNotFound(output_id.clone()))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let new_datum = output
        .typed_datum()
        .ok_or(CheckingAccountError::DatumNotFoundForOutput(output_id))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let redeemer = ();
    let script = Box::new(checking_account_validator);
    let mut values = output.values().to_owned();
    values.add_one_value(&PolicyId::Lovelace, amount);
    let actions = TxActions::v2()
        .with_script_redeem(output, redeemer, script)
        .with_script_init(new_datum, values, checking_account_address);
    Ok(actions)
}
