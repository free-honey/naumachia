use crate::address::ValidAddress;
use crate::{
    error::{Error, Result},
    output::Output,
    scripts::TxContext,
    PolicyId, Transaction,
};
use std::{fmt::Debug, hash::Hash};

pub fn can_spend_inputs<
    Address: ValidAddress,
    Datum: Clone + PartialEq + Debug,
    Redeemer: Clone + PartialEq + Eq + Hash,
>(
    tx: &Transaction<Address, Datum, Redeemer>,
    signer: Address,
) -> Result<()> {
    let ctx = TxContext { signer };
    for input in &tx.inputs {
        match input {
            Output::Wallet { .. } => {} // TODO: Make sure not spending other's outputs
            Output::Validator { owner, datum, .. } => {
                let script = tx
                    .validators
                    .get(owner)
                    .ok_or_else(|| Error::FailedToRetrieveScriptFor(owner.to_owned().into()))?;
                let (_, redeemer) = tx
                    .redeemers
                    .iter()
                    .find(|(utxo, _)| utxo == input)
                    .ok_or_else(|| Error::FailedToRetrieveRedeemerFor(owner.to_owned().into()))?;

                script.execute(datum.clone(), redeemer.clone(), ctx.clone())?;
            }
        }
    }
    Ok(())
}

pub fn can_mint_tokens<Address: ValidAddress, Datum, Redeemer>(
    tx: &Transaction<Address, Datum, Redeemer>,
    signer: &Address,
) -> Result<()> {
    let ctx = TxContext {
        signer: signer.clone(),
    };
    for (id, _) in tx.minting.as_iter() {
        match id {
            PolicyId::NativeToken(_) => {
                if let Some(policy) = tx.policies.get(id) {
                    policy.execute(ctx.clone())?;
                } else {
                    return Err(Error::FailedToRetrievePolicyFor(id.to_owned()));
                }
            }
            PolicyId::ADA => {
                return Err(Error::ImpossibleToMintADA);
            }
        }
    }
    Ok(())
}
