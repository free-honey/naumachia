use context.{OutputReference, Input}

pub fn ref_matches_input_in_list(ref: OutputReference, inputs: List<Input>) -> Bool {
    when inputs is {
        [input, ..tail] -> if ref == input.output_reference {
                True
            } else {
                ref_matches_input_in_list(ref, tail)
        }
        [] -> False
    }
}

