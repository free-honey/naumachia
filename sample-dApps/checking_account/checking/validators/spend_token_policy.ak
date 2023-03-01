use aiken/list.{has}
use aiken/transaction.{ScriptContext}
use aiken/transaction/value.{PolicyId}

validator mint(checking_account_nft: PolicyId, owner: ByteArray) {
  fn(_redeemer: Void, ctx: ScriptContext) -> Bool {
    has(ctx.transaction.extra_signatories, owner)
  }
}
