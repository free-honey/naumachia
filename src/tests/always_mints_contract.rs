use crate::tests::MockBackend;
use crate::{error, Address, DataSource, Output, SmartContract, UnBuiltTransaction};
use std::cell::RefCell;
use std::collections::HashMap;

use error::Result;

struct AlwaysMintsSmartContract;

enum Endpoint {
    Mint {
        amount: u64, // TODO: Too big?
                     // recipient: Option<Address>
    },
}

const MINT_POLICY_ADDR: &str = "mint_policy";

impl SmartContract for AlwaysMintsSmartContract {
    type Endpoint = Endpoint;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        _source: &D,
    ) -> Result<UnBuiltTransaction> {
        match endpoint {
            Endpoint::Mint { amount } => mint(amount),
        }
    }
}

fn mint(amount: u64) -> Result<UnBuiltTransaction> {
    let mut value = HashMap::new();
    let address = Address::new(MINT_POLICY_ADDR);
    value.insert(Some(address), amount);
    let output = Output { value };
    let utx = UnBuiltTransaction {
        inputs: vec![],
        outputs: vec![output],
    };
    Ok(utx)
}

#[test]
fn can_mint_from_always_true_minting_policy() {
    let backend = MockBackend {
        me: Address::new("me"),
        outputs: RefCell::new(vec![]),
    };
    // Call mint endpoint
    let amount = 69;
    let call = Endpoint::Mint { amount };
    AlwaysMintsSmartContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();
    // Wait 1 block? IDK if we need to wait. That's an implementation detail of a specific data
    // source I think? Could be wrong.

    // Check my balance for minted tokens
    let expected = amount;
    let actual = backend.my_balance(&Some(Address::new(MINT_POLICY_ADDR))); // TODO: Use policy address
    assert_eq!(expected, actual)
}
