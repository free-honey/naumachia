use aiken/transaction.{ScriptContext, Input}
use aiken/transaction/value.{PolicyId, Value, policies}
use aiken/list.{any}

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

pub fn spend(spend_token_policy: PolicyId, _datum, _redeemer, ctx: ScriptContext) -> Bool {
    ctx_contains_spend_token(ctx, spend_token_policy)
}