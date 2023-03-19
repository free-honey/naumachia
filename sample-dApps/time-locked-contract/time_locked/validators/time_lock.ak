use aiken/transaction.{ScriptContext}
use interval.{contains_interval, lower_bound_range}
use timestamp.{Timestamp}

validator {
    fn spend(datum: Int, _redeemer: Void, ctx: ScriptContext) -> Bool {
        let tx_range = ctx.transaction.validity_range
        let datum_range = lower_bound_range(datum, True)
        contains_interval(datum_range, tx_range)    
    }
}
