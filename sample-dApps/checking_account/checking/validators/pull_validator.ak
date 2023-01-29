use aiken/transaction.{ScriptContext}

type Datum {
    next_pull: Int,
}

pub fn spend(_datum, _redeemer, ctx: ScriptContext) -> Bool {
    // This Script
    // 1. Check that it's past date
    // 2. Check taht new datum is made
    //    a. Datum has updated date (old_date + period)
    //    b. Everything else matches
    // 3. Check that datum is at current address
    // 4. Check that UTxO includes spending token!
    // That Script
    // 1. Check that Checking Account UTxO is replaced with Current Balance - Pull Amount
    // 2. Check that CA datum on new Output && matches
    // 3. Check that CA NFT on CA Output
    True
}