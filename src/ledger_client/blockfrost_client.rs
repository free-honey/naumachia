use crate::ledger_client::{LedgerClient, TxORecordResult};
use crate::output::Output;
use crate::{Address, Transaction};
use std::marker::PhantomData;

pub mod blockfrost_http_client;

pub mod keys;

pub struct BlockFrostLedgerClient<Datum, Redeemer> {
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

impl<Datum, Redeemer> LedgerClient<Datum, Redeemer> for BlockFrostLedgerClient<Datum, Redeemer> {
    fn signer(&self) -> &Address {
        todo!()
    }

    fn outputs_at_address(&self, _address: &Address) -> Vec<Output<Datum>> {
        todo!()
    }

    fn issue(&self, _tx: Transaction<Datum, Redeemer>) -> TxORecordResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger_client::blockfrost_client::{
        blockfrost_http_client::{tests::get_test_bf_http_clent, BlockfrostHttp},
        keys::{base_address_from_entropy, load_phrase_from_file, TESTNET},
    };
    use bip39::{Language, Mnemonic};
    use cardano_multiplatform_lib::address::{BaseAddress, RewardAddress};
    use futures::future::{join_all, select_all};

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
        // Should be: stake_test1urmk5g3m8wstqzgqfhqgyl4gqn2jpusz8wa96d5y4kdrg5cvf83m6
        let staking_cred = base_addr.stake_cred();

        let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
            .to_address()
            .to_bech32(None)
            .unwrap();
        dbg!(&reward_addr);

        let bf = get_test_bf_http_clent();

        let addresses = bf.assoc_addresses(&reward_addr).await.unwrap();

        dbg!(&addresses);

        let utxos_futs: Vec<_> = addresses
            .iter()
            .map(|addr| bf.utxos(addr.address()))
            .collect();

        let utxos: Vec<_> = join_all(utxos_futs)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .flatten()
            .collect();

        dbg!(&utxos);
    }
}
