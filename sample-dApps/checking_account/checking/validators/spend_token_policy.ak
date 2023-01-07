use aiken/transaction.{ScriptContext}
use aiken/transaction/value.{PolicyId}

pub fn mint(checking_account_nft: PolicyId, _redeemer, ctx: ScriptContext) -> Bool {
    True
}