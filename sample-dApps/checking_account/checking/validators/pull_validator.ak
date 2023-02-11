use aiken/transaction.{ScriptContext}
use aiken/interval.{IntervalBoundType, Finite}

pub type Datum {
    next_pull: Int,
}

fn past_next_pull_date(datum: Datum, ctx: ScriptContext) -> Bool {
    let next_pull = datum.next_pull
    let is_inclusive = ctx.transaction.validity_range.lower_bound.is_inclusive
    when ctx.transaction.validity_range.lower_bound.bound_type is {
        Finite(lower) -> {
            if is_inclusive { next_pull <= lower } else { next_pull < lower }
        }
        _ -> False
    }
}

fn only_one_spend_token(datum: Datum, ctx: ScriptContext) -> Bool {
    True
}

pub fn spend(datum: Datum, _redeemer: Void, ctx: ScriptContext) -> Bool {
    // This Script
    // 1. Check that it's past date
    past_next_pull_date(datum, ctx) &&
    // 2. Check taht new datum is made
    //    a. Datum has updated date (old_date + period)
    //    b. Everything else matches
    // 3. Check that datum is at current address
    // 4. Check that UTxO includes spending token!
    //  5. Only one spend token -- actually, should this be on the checking account I think
    // That Script
    // 1. Check that Checking Account UTxO is replaced with Current Balance - Pull Amount
    // 2. Check that CA datum on new Output && matches
    // 3. Check that CA NFT on CA Output
    True
}