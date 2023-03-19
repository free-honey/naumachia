use aiken/list.{has}
use aiken/transaction.{ScriptContext}
use aiken/transaction/value.{PolicyId}

validator(checking_account_nft: PolicyId, owner: ByteArray) {
  fn mint(_redeemer: Void, ctx: ScriptContext) -> Bool {
    has(ctx.transaction.extra_signatories, owner)
  }
}
