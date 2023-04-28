use aiken/transaction.{ScriptContext, OutputReference, Output, Input, Spend, InlineDatum}
use aiken/transaction/credential.{Address}
use aiken/transaction/value
use aiken/interval.{IntervalBoundType, Finite}
use aiken/list
use aiken/option
use checking/datum.{CheckingAccountDatum}

pub type PullDatum {
    owner: ByteArray,
    puller: ByteArray,
    amount_lovelace: Int,
    next_pull: Int,
    period: Int,
    spend_token: ByteArray,
    checking_account_nft: ByteArray,
}

fn past_next_pull_date(datum: PullDatum, ctx: ScriptContext) -> Bool {
    let next_pull = datum.next_pull
    let is_inclusive = ctx.transaction.validity_range.lower_bound.is_inclusive
    when ctx.transaction.validity_range.lower_bound.bound_type is {
        Finite(lower) -> {
            if is_inclusive { next_pull <= lower } else { next_pull < lower }
        }
        _ -> False
    }
}

// This just chooses the first output at that address, behavior undefined if there are multiple
fn new_datum_is_made(datum: PullDatum, ctx: ScriptContext) -> Bool {
    let output_at_address = fn(address: Address) -> Option<Output> {
        let address_matches = fn(output: Output) -> Bool {
            output.address == address
        }
        ctx.transaction.outputs
        |> list.find(address_matches)
    }

    let checks_on_output = fn(output: Output, old_datum: PullDatum) -> Bool {
            expect InlineDatum(untyped_datum) = output.datum
            expect new_datum: PullDatum = untyped_datum
            //    a. New datum has updated date 
            new_datum.next_pull == old_datum.next_pull + old_datum.period &&
            //    b. Everything else matches 
            new_datum.puller == old_datum.puller &&
            new_datum.amount_lovelace == old_datum.amount_lovelace &&
            new_datum.period == old_datum.period &&
            new_datum.spend_token == old_datum.spend_token &&
            new_datum.checking_account_nft == old_datum.checking_account_nft &&
            //    d. Check that UTxO includes spending token!
            { 
                output.value
                |> value.policies
                |> list.any(fn(id) {id == old_datum.spend_token})
            }
    }

    expect Spend(out_ref) = ctx.purpose
    let inputs = ctx.transaction.inputs
    when find_input_address(out_ref, inputs)
    |> option.and_then(output_at_address) is {
        Some(output) -> checks_on_output(output, datum)
        None -> False
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
    |> list.find(output_ref_matches)
    |> option.map(address_from_output)
}

fn checking_account_checks(datum: PullDatum, ctx: ScriptContext) -> Bool {
    expect Some(input) = find_account_input(datum, ctx)
    let account_address = input.output.address
    expect Some(output) = find_account_output(datum, ctx, account_address)
    expect InlineDatum(input_datum) = input.output.datum
    expect input_account_datum: CheckingAccountDatum = input_datum
    expect InlineDatum(output_datum) = output.datum
    expect output_account_datum: CheckingAccountDatum = output_datum

    let input_lovelace = input.output.value
    |> value.lovelace_of

    let output_lovelace = output.value
    |> value.lovelace_of

    output_lovelace == input_lovelace - datum.amount_lovelace 
    && 
    input_account_datum == output_account_datum
}

fn find_account_input(datum: PullDatum, ctx: ScriptContext) -> Option<Input> {
    ctx.transaction.inputs
    |> list.find(fn(input) -> Bool {
        let list_length = input.output.value
        |> value.flatten()
        |> list.filter(fn(asset) -> Bool {
            let (policy_id, _, amt) = asset
            policy_id == datum.checking_account_nft &&
            amt == 1
        })
        |> list.length()

        list_length == 1
    })
}

fn find_account_output(datum: PullDatum, ctx: ScriptContext, account_address: Address) -> Option<Output> {
    ctx.transaction.outputs
    |> list.filter(fn(output) -> Bool {
        output.address == account_address
    })
    |> list.find(fn(output) -> Bool {
        let list_length = output.value
        |> value.flatten()
        |> list.filter(fn(asset) -> Bool {
            let (policy_id, _, amt) = asset
            policy_id == datum.checking_account_nft &&
            amt == 1
        })
        |> list.length()

        list_length == 1
    })
}

validator {
    fn spend(datum: PullDatum, _redeemer: Void, ctx: ScriptContext) -> Bool {
        let signed_by_owner = ctx.transaction.extra_signatories
        |> list.any(fn(signature) {
            signature == datum.owner
        }) 

        let signed_by_puller = ctx.transaction.extra_signatories
        |> list.any(fn(signature) {
            signature == datum.puller
        }) 
        
        signed_by_owner || {
        // This Script
        // 1. Signed by puller
        signed_by_puller &&
        // 2. Check that it's past date
        past_next_pull_date(datum, ctx) &&
        // 3. Check taht new datum is made
        new_datum_is_made(datum, ctx) &&
        // That Script
        // 4. Check that Checking Account UTxO is replaced with Current Balance - Pull Amount
        // 5. Check that CA datum on new Output && matches
        // 6. Check that CA NFT on CA Output
        checking_account_checks(datum, ctx)
        }
}
}