use crate::ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult};
use crate::values::Values;
use crate::{address, output::Output, Address, PolicyId, Transaction};
use async_trait::async_trait;
use blockfrost_http_client::keys::TESTNET;
use blockfrost_http_client::schemas::{UTxO, Value};
use blockfrost_http_client::BlockFrostHttpTrait;
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress, RewardAddress};
use futures::{future::join_all, FutureExt};
use std::marker::PhantomData;
use thiserror::Error;

pub struct BlockFrostLedgerClient<'a, Client: BlockFrostHttpTrait, Datum, Redeemer> {
    client: &'a Client,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<'a, Client, D, R> BlockFrostLedgerClient<'a, Client, D, R>
where
    Client: BlockFrostHttpTrait,
    D: Default,
    R: Default,
{
    pub fn new(client: &'a Client) -> Self {
        BlockFrostLedgerClient {
            client,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}
#[derive(Debug, Error)]
enum BFLCError {
    #[error("CML JsError: {0:?}")]
    JsError(String),
    #[error("Not a valid BaseAddress")]
    InvalidBaseAddr,
}

fn output_cml_err(
    address: &Address,
) -> impl Fn(cardano_multiplatform_lib::error::JsError) -> LedgerClientError + '_ {
    move |e| {
        LedgerClientError::FailedToRetrieveOutputsAt(
            address.to_owned(),
            Box::new(BFLCError::JsError(format!("{:?}", e))),
        )
    }
}

fn output_http_err(
    address: &Address,
) -> impl Fn(blockfrost_http_client::error::Error) -> LedgerClientError + '_ {
    move |e| LedgerClientError::FailedToRetrieveOutputsAt(address.to_owned(), Box::new(e))
}

fn invalid_base_addr(address: &Address) -> LedgerClientError {
    LedgerClientError::FailedToRetrieveOutputsAt(
        address.to_owned(),
        Box::new(BFLCError::InvalidBaseAddr),
    )
}

#[async_trait]
impl<Client, Datum, Redeemer> LedgerClient<Datum, Redeemer>
    for BlockFrostLedgerClient<'_, Client, Datum, Redeemer>
where
    Client: BlockFrostHttpTrait + Send + Sync,
    Datum: Send + Sync,
    Redeemer: Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<&Address> {
        todo!()
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match address {
            Address::Base(addr_string) => {
                let cml_address =
                    CMLAddress::from_bech32(addr_string).map_err(output_cml_err(address))?;
                let base_addr = BaseAddress::from_address(&cml_address)
                    .ok_or_else(|| invalid_base_addr(address))?;
                let staking_cred = base_addr.stake_cred();

                let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
                    .to_address()
                    .to_bech32(None)
                    .map_err(output_cml_err(address))?;

                let addresses = self
                    .client
                    .assoc_addresses(&reward_addr)
                    .await
                    .map_err(output_http_err(address))?;

                let nested_utxos_futs: Vec<_> = addresses
                    .iter()
                    .map(|addr| {
                        self.client
                            .utxos(addr.as_string())
                            .map(|utxos| (addr.to_owned(), utxos))
                    })
                    .collect();

                let nested_utxos: Vec<_> = join_all(nested_utxos_futs).await;

                let mut outputs_for_all_addresses = Vec::new();

                for (addr, utxos_res) in nested_utxos {
                    let utxos = utxos_res.map_err(output_http_err(address))?;
                    let nau_addr = convert_address(addr);
                    let nau_outputs: Vec<_> = utxos
                        .iter()
                        .map(|utxo| into_nau_output(utxo, &nau_addr))
                        .collect();
                    outputs_for_all_addresses.extend(nau_outputs);
                }
                Ok(outputs_for_all_addresses)
            }
            Address::Raw(_) => unimplemented!("Doesn't make sense here"),
        }
    }

    fn issue(&self, _tx: Transaction<Datum, Redeemer>) -> LedgerClientResult<()> {
        todo!()
    }
}

fn into_nau_output<Datum>(utxo: &UTxO, owner: &address::Address) -> Output<Datum> {
    let tx_hash = utxo.tx_hash().to_owned();
    let index = utxo.output_index().to_owned();
    let mut values = Values::default();
    utxo.amount()
        .iter()
        .map(|value| as_nau_value(value))
        .for_each(|(policy_id, amount)| values.add_one_value(&policy_id, amount));
    Output::new_wallet(tx_hash, index, owner.to_owned(), values)
}

fn as_nau_value(value: &Value) -> (PolicyId, u64) {
    let policy_id = match value.unit() {
        "lovelace" => PolicyId::ADA,
        native_token => {
            let policy = &native_token[..56]; // TODO: Use the rest as asset info
            PolicyId::native_token(policy)
        }
    };
    let amount = value.quantity().parse().unwrap(); // TODO: unwrap
    (policy_id, amount)
}

fn convert_address(bf_addr: blockfrost_http_client::schemas::Address) -> Address {
    Address::new(bf_addr.as_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PolicyId;
    use blockfrost_http_client::{get_test_bf_http_client, keys::my_base_addr};

    #[ignore]
    #[tokio::test]
    async fn get_all_my_utxos() {
        let base_addr = my_base_addr();
        let addr_string = base_addr.to_address().to_bech32(None).unwrap();
        let my_addr = Address::Base(addr_string);

        let client = get_test_bf_http_client().unwrap();

        let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

        let my_utxos = bf.outputs_at_address(&my_addr).await.unwrap();

        dbg!(my_utxos);
    }

    #[ignore]
    #[tokio::test]
    async fn get_my_lovelace_balance() {
        let base_addr = my_base_addr();
        let addr_string = base_addr.to_address().to_bech32(None).unwrap();
        let my_addr = Address::Base(addr_string);

        let client = get_test_bf_http_client().unwrap();

        let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

        let my_balance = bf
            .balance_at_address(&my_addr, &PolicyId::ADA)
            .await
            .unwrap();

        println!();
        println!("ADA: {:?}", my_balance);
    }

    #[ignore]
    #[tokio::test]
    async fn get_my_native_token_balance() {
        let base_addr = my_base_addr();
        let addr_string = base_addr.to_address().to_bech32(None).unwrap();
        let my_addr = Address::Base(addr_string);

        let client = get_test_bf_http_client().unwrap();

        let bf = BlockFrostLedgerClient::<_, (), ()>::new(&client);

        let policy =
            PolicyId::native_token("57fca08abbaddee36da742a839f7d83a7e1d2419f1507fcbf3916522");
        let my_balance = bf.balance_at_address(&my_addr, &policy).await.unwrap();

        println!();
        println!("Native Token {:?}: {:?}", policy, my_balance);
    }
}
