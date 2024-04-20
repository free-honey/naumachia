use crate::{
    checking_account_validator,
    pull_validator,
    CheckingAccountDatums,
    CheckingAccountError,
};
use naumachia::{
    ledger_client::LedgerClient,
    logic::error::{
        SCLogicError,
        SCLogicResult,
    },
    output::{
        Output,
        OutputId,
    },
    policy_id::PolicyId,
    scripts::Validator,
    transaction::TxActions,
    values::Values,
};

pub async fn pull_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    allow_pull_output_id: OutputId,
    checking_account_output_id: OutputId,
) -> SCLogicResult<TxActions<CheckingAccountDatums, ()>> {
    let network = ledger_client.network().await?;
    let allow_pull_validator = pull_validator()?;
    let allow_pull_address = allow_pull_validator.address(network)?;
    let allow_pull_output = ledger_client
        .all_outputs_at_address(&allow_pull_address)
        .await?
        .into_iter()
        .find(|o| o.id() == &allow_pull_output_id)
        .ok_or(CheckingAccountError::OutputNotFound(
            allow_pull_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let validator = checking_account_validator()?;
    let checking_account_address = validator.address(network)?;
    let checking_account_output = ledger_client
        .all_outputs_at_address(&checking_account_address)
        .await?
        .into_iter()
        .find(|o| o.id() == &checking_account_output_id)
        .ok_or(CheckingAccountError::OutputNotFound(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let old_allow_pull_datum = allow_pull_output
        .typed_datum()
        .ok_or(CheckingAccountError::DatumNotFoundForOutput(
            allow_pull_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let allow_pull_redeemer = ();
    let allow_pull_script = Box::new(allow_pull_validator);
    let allow_pull_value = allow_pull_output.values().clone();

    let (old_allow_pull_datum, new_allow_pull_datum) =
        if let CheckingAccountDatums::AllowedPuller(old) = old_allow_pull_datum {
            let new = old.next();
            (old, new)
        } else {
            unimplemented!()
        };
    let amount = old_allow_pull_datum.amount_lovelace;

    let old_pull_time = old_allow_pull_datum.next_pull;
    let valid_pull_time = valid_time_to_pull(ledger_client, old_pull_time).await?;

    let checking_account_redeemer = ();
    let checking_account_script = Box::new(validator);

    let new_checking_account_datum = get_datum_from_output(&checking_account_output)?;
    let new_account_value =
        calculate_new_account_value(&checking_account_output, amount)?;

    let actions = TxActions::v2()
        .with_script_redeem(allow_pull_output, allow_pull_redeemer, allow_pull_script)
        .with_script_init(
            new_allow_pull_datum.into(),
            allow_pull_value,
            allow_pull_address,
        )
        .with_script_redeem(
            checking_account_output,
            checking_account_redeemer,
            checking_account_script,
        )
        .with_script_init(
            new_checking_account_datum,
            new_account_value,
            checking_account_address,
        )
        .with_valid_range_secs(Some(valid_pull_time / 1000), None);
    Ok(actions)
}

fn calculate_new_account_value(
    checking_account_output: &Output<CheckingAccountDatums>,
    subtract: u64,
) -> SCLogicResult<Values> {
    let old_values = checking_account_output.values().to_owned();
    let mut sub_values = Values::default();
    sub_values.add_one_value(&PolicyId::Lovelace, subtract);
    let new_account_values = old_values
        .try_subtract(&sub_values)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;
    Ok(new_account_values)
}

fn get_datum_from_output(
    output: &Output<CheckingAccountDatums>,
) -> SCLogicResult<CheckingAccountDatums> {
    output
        .typed_datum()
        .ok_or(CheckingAccountError::DatumNotFoundForOutput(
            output.id().clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))
        .map(|d| d.clone())
}

async fn valid_time_to_pull<LC>(
    ledger_client: &LC,
    old_pull_time: i64,
) -> SCLogicResult<i64>
where
    LC: LedgerClient<CheckingAccountDatums, ()>,
{
    let current_time = ledger_client
        .current_time_secs()
        .await
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    if current_time < old_pull_time {
        let err = CheckingAccountError::TooEarlyToPull {
            next_pull: old_pull_time,
            current_time,
        };
        return Err(SCLogicError::Endpoint(Box::new(err)));
    }

    Ok(current_time)
}
