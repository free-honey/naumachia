use naumachia::scripts::raw_policy_script::TwoParamRawPolicy;
use naumachia::scripts::raw_script::PlutusScriptFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::{ScriptError, ScriptResult};

const SCRIPT_RAW: &str =
    include_str!("../../checking/assets/spend_token_policy/mint/payment_script.json");

pub struct CheckingAccountNFT {
    inner: Vec<u8>,
}

impl From<CheckingAccountNFT> for PlutusData {
    fn from(value: CheckingAccountNFT) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub struct Owner {
    inner: Vec<u8>,
}

impl From<Owner> for PlutusData {
    fn from(value: Owner) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub fn spend_token_policy() -> ScriptResult<TwoParamRawPolicy<CheckingAccountNFT, Owner, ()>> {
    let script_file: PlutusScriptFile = serde_json::from_str(SCRIPT_RAW)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let raw_script_validator = TwoParamRawPolicy::new_v2(script_file)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::address::Address;
    use naumachia::scripts::context::ContextBuilder;
    use naumachia::scripts::MintingPolicy;

    #[test]
    fn execute__correct_signer_can_mint() {
        let signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let param_script = spend_token_policy().unwrap();
        let nft = CheckingAccountNFT {
            inner: vec![1, 2, 3],
        };
        let owner = Owner {
            inner: signer.bytes().unwrap().to_vec(),
        };
        let script = param_script.apply(nft).unwrap().apply(owner).unwrap();

        let ctx = ContextBuilder::new(signer).build();

        script.execute((), ctx).unwrap();
    }

    #[test]
    fn execute__incorrect_signer_cannot_mint() {
        let correct_signer = Address::new("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr");
        let incorrect_signer = Address::new("addr_test1qqddk5xnz08mxsqw6jdaenvhdah835lhvm62tt5lydk2as7kfjf77qy57hqhnefcqyy7hmhsygj9j38rj984hn9r57fs066hcl");
        let param_script = spend_token_policy().unwrap();
        let nft = CheckingAccountNFT {
            inner: vec![1, 2, 3],
        };
        let owner = Owner {
            inner: correct_signer.bytes().unwrap().to_vec(),
        };
        let script = param_script.apply(nft).unwrap().apply(owner).unwrap();

        let ctx = ContextBuilder::new(incorrect_signer).build();

        script.execute((), ctx).unwrap_err();
    }
}
