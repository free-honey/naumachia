use crate::CheckingAccountDatums;
use naumachia::address::Address;
use naumachia::scripts::context::TxContext;
use naumachia::scripts::{ScriptResult, ValidatorCode};

pub mod checking_account_validtor;
pub mod spend_token_policy;

// // TODO: Parameterize by some Policy whose tokens are allowed to access and owner
// pub struct FakeCheckingAccountValidator;
//
// impl ValidatorCode<CheckingAccountDatums, ()> for FakeCheckingAccountValidator {
//     fn execute(
//         &self,
//         _datum: CheckingAccountDatums,
//         _redeemer: (),
//         _ctx: TxContext,
//     ) -> ScriptResult<()> {
//         Ok(())
//     }
//
//     fn address(&self, _network: u8) -> ScriptResult<Address> {
//         let address =
//             Address::new("addr_test1wpz47fyj9ffqck2fnld04k27zfe04wq6n9zj76u4ghu4xdck8aqy7");
//         Ok(address)
//     }
//
//     fn script_hex(&self) -> ScriptResult<String> {
//         todo!()
//     }
// }

pub struct FakePullerValidator;

impl ValidatorCode<CheckingAccountDatums, ()> for FakePullerValidator {
    fn execute(
        &self,
        _datum: CheckingAccountDatums,
        _redeemer: (),
        _ctx: TxContext,
    ) -> ScriptResult<()> {
        Ok(())
    }

    fn address(&self, _network: u8) -> ScriptResult<Address> {
        let address =
            Address::new("addr_test1wzl0nd2r9mnegmqr2ec4pjpa5kktgt56r7zzp7fds5p0sac8atl7r");
        Ok(address)
    }

    fn script_hex(&self) -> ScriptResult<String> {
        todo!()
    }
}
