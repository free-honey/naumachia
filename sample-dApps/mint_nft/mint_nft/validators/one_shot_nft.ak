use context.{ScriptContext, OutputReference, Input}
use context/mint.{Mint}
use lists.{any_in_list}
use inputs.{ref_matches_input_in_list}

pub fn mint(input_ref: OutputReference, _redeemer, ctx: ScriptContext<mint.Mint>) -> Bool {
    let inputs = ctx.transaction.inputs
    let ref_matches_input = fn(input: Input) {
        input.output_reference == input_ref
    }
    any_in_list(inputs, ref_matches_input)
}
