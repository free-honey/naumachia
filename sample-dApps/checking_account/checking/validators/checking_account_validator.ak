use aiken/transaction.{ScriptContext, Input}
use aiken/transaction/value.{PolicyId, Value, policies}
use aiken/list.{any}

pub type CheckingAccountDatum {
    owner: ByteArray,
    spend_token_policy: PolicyId,
}

fn ctx_contains_spend_token(ctx: ScriptContext, spend_token_policy: PolicyId) -> Bool {
    let equals_policy = fn(policy) {
        policy == spend_token_policy
    }
    let input_contains_spend_token = fn(input: Input) {
        let policies = policies(input.output.value)
        any(policies, equals_policy)
    }
    any(ctx.transaction.inputs, input_contains_spend_token)
}

fn ctx_contains_owner_signature(ctx: ScriptContext, owner: ByteArray) -> Bool {
    let equals_owner = fn(signature) {
        signature == owner
    }

    any(ctx.transaction.extra_signatories, equals_owner)
}

pub fn spend(datum: CheckingAccountDatum, _redeemer: Void, ctx: ScriptContext) -> Bool {
    ctx_contains_spend_token(ctx, datum.spend_token_policy)
    || ctx_contains_owner_signature(ctx, datum.owner)
}