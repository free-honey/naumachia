use aiken/transaction.{ScriptContext}
use aiken/transaction/value.{PolicyId}
use aiken/list.{has}

pub fn mint(checking_account_nft: PolicyId, owner: ByteArray, _redeemer: Void, ctx: ScriptContext) -> Bool {
    has(ctx.transaction.extra_signatories, owner)
}