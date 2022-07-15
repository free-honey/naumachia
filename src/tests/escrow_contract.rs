use crate::tests::MockBackend;
use crate::validator::{TxContext, ValidatorCode};
use crate::{Address, DataSource, Output, SmartContract, UnBuiltTransaction, ADA};
use std::cell::RefCell;
use std::collections::HashMap;

// {-# INLINABLE meetsTarget #-}
// meetsTarget :: TxInfo -> EscrowTarget DatumHash -> Bool
// meetsTarget ptx = \case
//     PaymentPubKeyTarget pkh vl ->
//         valuePaidTo ptx (unPaymentPubKeyHash pkh) `geq` vl
//     ScriptTarget validatorHash dataValue vl ->
//         case scriptOutputsAt validatorHash ptx of
//             [(dataValue', vl')] ->
//                 traceIfFalse "dataValue" (dataValue' == dataValue)
//                 && traceIfFalse "value" (vl' `geq` vl)
//             _ -> False
//
// {-# INLINABLE validate #-}
// validate :: EscrowParams DatumHash -> PaymentPubKeyHash -> Action -> ScriptContext -> Bool
// validate EscrowParams{escrowDeadline, escrowTargets} contributor action ScriptContext{scriptContextTxInfo} =
//     case action of
//         Redeem ->
//             traceIfFalse "escrowDeadline-after" (escrowDeadline `after` txInfoValidRange scriptContextTxInfo)
//             && traceIfFalse "meetsTarget" (all (meetsTarget scriptContextTxInfo) escrowTargets)
//         Refund ->
//             traceIfFalse "escrowDeadline-before" ((escrowDeadline - 1) `before` txInfoValidRange scriptContextTxInfo)
//             && traceIfFalse "txSignedBy" (scriptContextTxInfo `txSignedBy` unPaymentPubKeyHash contributor)
struct EscrowValidatorScript;

impl ValidatorCode for EscrowValidatorScript {
    fn execute<D, R>(datum: D, redeemer: R, ctx: TxContext) -> bool {
        todo!()
    }

    fn address() -> Address {
        todo!()
    }
}

struct EscrowContract;

enum Endpoint {
    Escrow { amount: u64 },
}

impl SmartContract for EscrowContract {
    type Endpoint = Endpoint;

    fn handle_endpoint<D: DataSource>(
        endpoint: Self::Endpoint,
        source: &D,
    ) -> crate::Result<UnBuiltTransaction> {
        match endpoint {
            Endpoint::Escrow { amount } => escrow(amount),
        }
    }
}

fn escrow(amount: u64) -> crate::Result<UnBuiltTransaction> {
    todo!()
}

#[test]
fn escrow__can_create_instance() {
    let mut value = HashMap::new();
    let amount = 4;
    value.insert(ADA, amount);
    let output = Output { value };
    let me = Address::new("me");
    let backend = MockBackend {
        me: me.clone(),
        outputs: RefCell::new(vec![(me, output)]),
    };
    // Call mint endpoint
    let call = Endpoint::Escrow { amount };
    EscrowContract::hit_endpoint(call, &backend, &backend, &backend).unwrap();
    // Wait 1 block? IDK if we need to wait. That's an implementation detail of a specific data
    // source I think? Could be wrong.

    // Check my balance for minted tokens
    let address = EscrowValidatorScript::address();
    let expected = amount;
    let actual = backend.balance_at_address(&address, &ADA);
    assert_eq!(expected, actual)
}
