use crate::{
    checking_account_validator, pull_validator, AllowedPuller, CheckingAccountDatums,
    CheckingAccountError,
};
use naumachia::{
    address::PolicyId,
    ledger_client::LedgerClient,
    logic::{SCLogicError, SCLogicResult},
    output::OutputId,
    scripts::ValidatorCode,
    transaction::TxActions,
    values::Values,
};

pub async fn pull_from_account<LC: LedgerClient<CheckingAccountDatums, ()>>(
    ledger_client: &LC,
    allow_pull_output_id: OutputId,
    checking_account_output_id: OutputId,
    amount: u64,
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

    #[allow(unused_assignments)]
    let old_pull_time;
    let new_allow_pull_datum = match old_allow_pull_datum {
        CheckingAccountDatums::AllowedPuller(old_allowed_puller) => {
            let AllowedPuller {
                next_pull, period, ..
            } = old_allowed_puller;
            old_pull_time = next_pull;
            let next_pull = old_pull_time + period;
            AllowedPuller {
                next_pull,
                ..old_allowed_puller
            }
            .into()
        }
        _ => {
            unimplemented!()
        }
    };

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

    let new_checking_account_datum = checking_account_output
        .typed_datum()
        .ok_or(CheckingAccountError::DatumNotFoundForOutput(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .clone();
    let checking_account_redeemer = ();
    let checking_account_script = Box::new(validator);

    let old_values = checking_account_output.values().to_owned();
    let mut sub_values = Values::default();
    sub_values.add_one_value(&PolicyId::Lovelace, amount);
    let new_account_value = old_values
        .try_subtract(&sub_values)
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?
        .ok_or(CheckingAccountError::OutputNotFound(
            checking_account_output_id.clone(),
        ))
        .map_err(|e| SCLogicError::Endpoint(Box::new(e)))?;

    let actions = TxActions::v2()
        .with_script_redeem(allow_pull_output, allow_pull_redeemer, allow_pull_script)
        .with_script_init(new_allow_pull_datum, allow_pull_value, allow_pull_address)
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
        .with_valid_range_secs(Some(current_time / 1000), None);
    Ok(actions)
}
