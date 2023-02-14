use aiken/transaction.{ScriptContext, OutputReference, Output, Input, Spend, InlineDatum}
use aiken/transaction/credential.{Address}
use aiken/interval.{IntervalBoundType, Finite}
use aiken/list.{find}
use aiken/option.{map, is_some, and_then}

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

fn new_datum_is_made(datum: Datum, ctx: ScriptContext) -> Bool {
    let output_at_address = fn(address: Address) -> Option<Output> {
        let address_matches = fn(output: Output) -> Bool {
            output.address == address
        }
        ctx.transaction.outputs
        |> find(address_matches)
    }

    let checks_on_output = fn(output: Output) -> Bool {
            expect InlineDatum(datum) = output.datum
            expect datum: Datum = datum
            //    a. Datum has updated date (old_date + period)
            //    b. Everything else matches
            //    c. Check that datum is at current address
            //    d. Check that UTxO includes spending token!
            True
    }

    when ctx.purpose is {
        Spend(out_ref) -> {
            let inputs = ctx.transaction.inputs
            when find_input_address(out_ref, inputs)
            |> and_then(output_at_address) is {
                Some(output) -> checks_on_output(output)
                None -> False
            }
        }
        _ -> False
    }

}

fn find_input_address(output_ref: OutputReference, outputs: List<Input>) -> Option<Address> {
    let output_ref_matches = fn(input: Input) -> Bool {
        input.output_reference == output_ref
    }
    let address_from_output = fn(input: Input) -> Address {
        input.output.address
    }
    outputs
    |> find(output_ref_matches)
    |> map(address_from_output)
}



pub fn spend(datum: Datum, _redeemer: Void, ctx: ScriptContext) -> Bool {
    // This Script
    // 1. Check that it's past date
    past_next_pull_date(datum, ctx) &&
    // 2. Check taht new datum is made
    new_datum_is_made(datum, ctx) &&
    // That Script
    // 1. Check that Checking Account UTxO is replaced with Current Balance - Pull Amount
    // 2. Check that CA datum on new Output && matches
    // 3. Check that CA NFT on CA Output
    True
}