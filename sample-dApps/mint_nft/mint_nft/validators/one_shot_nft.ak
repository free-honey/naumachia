use context.{ScriptContext, Input}
use context/mint.{Mint}
use lists.{list_contains_item}

pub fn mint(input: Input, _redeemer, ctx: ScriptContext(mint.Mint)) -> Bool {
    let inputs = ctx.transaction.inputs
    list_contains_item(inputs, input)
}
