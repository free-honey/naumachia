// use super::*;
//
// struct TransferADASmartContract;
//
// enum Endpoint {
//     Transfer { amount: u64, recipient: Address },
// }
//
// impl Logic for TransferADASmartContract {
//     type Endpoint = Endpoint;
//     type Datum = ();
//     type Redeemer = ();
//
//     fn handle_endpoint(
//         endpoint: Self::Endpoint,
//         _issuer: &Address,
//     ) -> Result<UnBuiltTransaction<(), ()>> {
//         match endpoint {
//             Endpoint::Transfer { amount, recipient } => {
//                 let u_tx = UnBuiltTransaction::default().with_transfer(amount, recipient, ADA);
//                 Ok(u_tx)
//             }
//         }
//     }
// }
//
// #[test]
// fn can_transfer_funds() {}
