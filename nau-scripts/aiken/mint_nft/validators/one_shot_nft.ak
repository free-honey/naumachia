use aiken/transaction.{ScriptContext, OutputReference, Input}
use aiken/list.{any}

validator mint(input_ref: OutputReference) {
    fn(_redeemer: Void, ctx: ScriptContext) -> Bool {
        let inputs = ctx.transaction.inputs
        let ref_matches_input = fn(input: Input) {
            input.output_reference == input_ref
        }
        any(inputs, ref_matches_input)
    }
}