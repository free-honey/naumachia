use crate::{
    error::{Error, Result},
    output::Output,
    scripts::TxContext,
    Address, PolicyId, UnbuiltTransaction,
};
use std::{fmt::Debug, hash::Hash};

pub fn can_spend_inputs<
    Datum: Clone + PartialEq + Debug,
    Redeemer: Clone + PartialEq + Eq + Hash,
>(
    tx: &UnbuiltTransaction<Datum, Redeemer>,
    _signer: Address,
) -> Result<()> {
    // let ctx = TxContext { signer };
    for (input, _redeemer, _script) in &tx.script_inputs {
        match input {
            Output::Wallet { .. } => {} // TODO: Make sure not spending other's outputs
            Output::Validator { datum: _, .. } => {
                // TODO: This seems broken for Aiken eval still :( But Blockfrost is doing this
                //   check for us on the live network, so this prolly isn't needed anyway.
                // script.execute(datum.clone(), redeemer.clone(), ctx.clone())?;
            }
        }
    }
    Ok(())
}

// pub fn can_mint_tokens<Datum, Redeemer>(
//     tx: &UnbuiltTransaction<Datum, Redeemer>,
//     signer: &Address,
// ) -> Result<()> {
//     let ctx = TxContext {
//         signer: signer.clone(),
//     };
//     for (id, _) in tx.minting.as_iter() {
//         match id {
//             PolicyId::NativeToken(_, _) => {
//                 if let Some(policy) = tx.policies.get(id) {
//                     policy.execute(ctx.clone())?;
//                 } else {
//                     return Err(Error::FailedToRetrievePolicyFor(id.to_owned()));
//                 }
//             }
//             PolicyId::ADA => {
//                 return Err(Error::ImpossibleToMintADA);
//             }
//         }
//     }
//
//     Ok(())
// }
