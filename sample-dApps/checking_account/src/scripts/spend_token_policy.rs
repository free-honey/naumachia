use naumachia::scripts::raw_policy_script::TwoParamRawPolicy;
use naumachia::scripts::raw_script::BlueprintFile;
use naumachia::scripts::raw_validator_script::plutus_data::PlutusData;
use naumachia::scripts::{ScriptError, ScriptResult};
use naumachia::Address;

const BLUEPRINT: &str = include_str!("../../checking/plutus.json");
const VALIDATOR_NAME: &str = "spend_token_policy";

pub struct CheckingAccountNFT {
    inner: Vec<u8>,
}

impl From<Vec<u8>> for CheckingAccountNFT {
    fn from(value: Vec<u8>) -> Self {
        CheckingAccountNFT { inner: value }
    }
}

impl From<CheckingAccountNFT> for PlutusData {
    fn from(value: CheckingAccountNFT) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub struct Owner {
    inner: Vec<u8>,
}

impl From<Address> for Owner {
    fn from(value: Address) -> Self {
        let inner = value.to_vec();
        Owner { inner }
    }
}

impl From<Owner> for PlutusData {
    fn from(value: Owner) -> Self {
        PlutusData::BoundedBytes(value.inner)
    }
}

pub fn spend_token_policy() -> ScriptResult<TwoParamRawPolicy<CheckingAccountNFT, Owner, ()>> {
    let script_file: BlueprintFile = serde_json::from_str(BLUEPRINT)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    let validator_blueprint =
        script_file
            .get_validator(VALIDATOR_NAME)
            .ok_or(ScriptError::FailedToConstruct(format!(
                "Validator not listed in Blueprint: {:?}",
                VALIDATOR_NAME
            )))?;
    let raw_script_validator = TwoParamRawPolicy::from_blueprint(validator_blueprint)
        .map_err(|e| ScriptError::FailedToConstruct(e.to_string()))?;
    Ok(raw_script_validator)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use naumachia::scripts::context::{pub_key_has_from_address_if_available, ContextBuilder};
    use naumachia::scripts::MintingPolicy;

    #[test]
    fn execute__correct_signer_can_mint() {
        let signer = Address::from_bech32("addr_test1qpmtp5t0t5y6cqkaz7rfsyrx7mld77kpvksgkwm0p7en7qum7a589n30e80tclzrrnj8qr4qvzj6al0vpgtnmrkkksnqd8upj0").unwrap();
        let param_script = spend_token_policy().unwrap();
        let nft = CheckingAccountNFT {
            inner: vec![1, 2, 3],
        };
        let signer_pkh = pub_key_has_from_address_if_available(&signer).unwrap();
        let owner = Owner {
            inner: signer_pkh.bytes(),
        };
        let script = param_script.apply(nft).unwrap().apply(owner).unwrap();

        let ctx = ContextBuilder::new(signer_pkh).build_mint(&[]);

        script.execute((), ctx).unwrap();
    }

    #[test]
    fn execute__incorrect_signer_cannot_mint() {
        let correct_signer = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
        let incorrect_signer = Address::from_bech32("addr_test1qqddk5xnz08mxsqw6jdaenvhdah835lhvm62tt5lydk2as7kfjf77qy57hqhnefcqyy7hmhsygj9j38rj984hn9r57fs066hcl").unwrap();
        let param_script = spend_token_policy().unwrap();
        let nft = CheckingAccountNFT {
            inner: vec![1, 2, 3],
        };
        let signer_pkh = pub_key_has_from_address_if_available(&correct_signer).unwrap();
        let owner = Owner {
            inner: signer_pkh.bytes(),
        };
        let script = param_script.apply(nft).unwrap().apply(owner).unwrap();
        let incorrect_signer_pkh =
            pub_key_has_from_address_if_available(&incorrect_signer).unwrap();
        let ctx = ContextBuilder::new(incorrect_signer_pkh).build_mint(&[]);

        script.execute((), ctx).unwrap_err();
    }
}
