// TODO: As of now, wallet, script, and policy addresses are the same. This is an
//       over-simplification in many ways. Wallet and address also need to be disambiguated.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Address(String);

impl Address {
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}

pub type Policy = Option<Address>;

pub const ADA: Policy = None;
