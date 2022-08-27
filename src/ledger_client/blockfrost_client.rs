use crate::ledger_client::blockfrost_client::blockfrost_http_client::{
    get_test_bf_http_clent, schemas,
};
use crate::{
    ledger_client::{
        blockfrost_client::{blockfrost_http_client::schemas::UTxO, keys::TESTNET},
        LedgerClient, TxORecordResult,
    },
    output::Output,
    Address, Transaction,
};
use async_trait::async_trait;
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress, RewardAddress};
use futures::{executor::block_on, future::join_all, FutureExt};
use std::marker::PhantomData;

pub mod blockfrost_http_client;

pub mod keys;

pub struct BlockFrostLedgerClient<Datum, Redeemer> {
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<D, R> BlockFrostLedgerClient<D, R> {
    pub fn new() -> Self {
        BlockFrostLedgerClient {
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}

#[async_trait]
impl<Datum: Send + Sync, Redeemer: Send + Sync> LedgerClient<Datum, Redeemer>
    for BlockFrostLedgerClient<Datum, Redeemer>
{
    fn signer(&self) -> &Address {
        todo!()
    }

    async fn outputs_at_address(&self, address: &Address) -> Vec<Output<Datum>> {
        match address {
            Address::Base(addr_string) => {
                let address = CMLAddress::from_bech32(addr_string).unwrap(); // TODO: unwrap
                let base_addr = BaseAddress::from_address(&address).unwrap(); // TODO: unwrap
                let staking_cred = base_addr.stake_cred();

                let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
                    .to_address()
                    .to_bech32(None)
                    .unwrap(); // TODO: unwrap
                dbg!(&reward_addr);

                let bf = get_test_bf_http_clent();

                let addresses = bf.assoc_addresses(&reward_addr).await.unwrap(); // TODO: unwrap

                dbg!(&addresses);

                let nested_utxos_futs: Vec<_> = addresses
                    .iter()
                    .map(|addr| {
                        bf.utxos(addr.as_string())
                            .map(|utxos| (addr.to_owned(), utxos))
                    })
                    .collect();

                let nested_utxos: Vec<_> = join_all(nested_utxos_futs).await;

                let mut outputs_for_all_addresses = Vec::new();

                for (addr, utxos_res) in nested_utxos {
                    let utxos = utxos_res.unwrap(); // TODO: unwrap
                    let nau_addr = addr.into();
                    let nau_outputs: Vec<_> = utxos
                        .iter()
                        .map(|utxo| into_nau_output(utxo, &nau_addr))
                        .collect();
                    outputs_for_all_addresses.extend(nau_outputs);
                }
                outputs_for_all_addresses
            }
            Address::Raw(_) => unimplemented!("Doesn't make sense here"),
        }
    }

    fn issue(&self, _tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()> {
        todo!()
    }
}

fn into_nau_output<Datum>(utxo: &UTxO, owner: &Address) -> Output<Datum> {
    todo!()
    // Output::new_wallet()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger_client::blockfrost_client::{
        blockfrost_http_client::BlockfrostHttp,
        keys::{base_address_from_entropy, load_phrase_from_file, TESTNET},
    };
    use bip39::{Language, Mnemonic};

    const CONFIG_PATH: &str = ".blockfrost.toml";

    pub fn my_base_addr() -> BaseAddress {
        let phrase = load_phrase_from_file(CONFIG_PATH);
        let mnemonic = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

        let entropy = mnemonic.entropy();

        base_address_from_entropy(&entropy, TESTNET)
    }

    #[ignore]
    #[tokio::test]
    async fn get_all_my_utxos() {
        let base_addr = my_base_addr();
        let addr_string = base_addr.to_address().to_bech32(None).unwrap();
        let my_addr = Address::Base(addr_string);

        let bf = BlockFrostLedgerClient::<(), ()>::new();

        let my_utxos = bf.outputs_at_address(&my_addr).await;

        dbg!(my_utxos);

        // Should be: stake_test1urmk5g3m8wstqzgqfhqgyl4gqn2jpusz8wa96d5y4kdrg5cvf83m6

        // let staking_cred = base_addr.stake_cred();
        //
        // let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
        //     .to_address()
        //     .to_bech32(None)
        //     .unwrap();
        // dbg!(&reward_addr);
        //
        // let bf = get_test_bf_http_clent();
        //
        // let addresses = bf.assoc_addresses(&reward_addr).await.unwrap();
        //
        // dbg!(&addresses);
        //
        // let utxos_futs: Vec<_> = addresses
        //     .iter()
        //     .map(|addr| bf.utxos(addr.address()))
        //     .collect();
        //
        // let utxos: Vec<_> = join_all(utxos_futs)
        //     .await
        //     .into_iter()
        //     .collect::<Result<Vec<_>, _>>()
        //     .unwrap()
        //     .into_iter()
        //     .flatten()
        //     .collect();
        //
        // dbg!(&utxos);
    }
}
