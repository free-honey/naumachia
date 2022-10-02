use crate::{
    error::*,
    ledger_client::{
        cml_client::{
            blockfrost_ledger::BlockFrostLedger, key_manager::KeyManager,
            plutus_data_interop::PlutusDataInterop, CMLLedgerCLient,
        },
        LedgerClient, LedgerClientResult,
    },
    output::Output,
    transaction::TxId,
    trireme_ledger_client::raw_secret_phrase::RawSecretPhraseKeys,
    Address, UnbuiltTransaction,
};
use async_trait::async_trait;
use dirs::home_dir;
use std::{marker::PhantomData, path::Path, path::PathBuf};

pub mod raw_secret_phrase;

pub const TRIREME_CONFIG_FOLDER: &str = ".trireme";

pub fn path_to_trireme_config_folder() -> Result<PathBuf> {
    let mut home_dir = home_dir().ok_or(Error::Trireme(
        "Could not find home directory :(".to_string(),
    ))?;
    home_dir.push(TRIREME_CONFIG_FOLDER);
    Ok(home_dir)
}

pub enum LedgerSource {
    BlockFrost,
}

pub enum KeySource {
    RawSecretPhrase { phrase_file: PathBuf },
}

pub struct TriremeConfig {
    ledger_source: LedgerSource,
    key_source: KeySource,
}

impl TriremeConfig {
    pub fn new(ledger_source: LedgerSource, key_source: KeySource) -> Self {
        TriremeConfig {
            ledger_source,
            key_source,
        }
    }

    pub fn to_client<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop>(
        self,
        network: u8,
    ) -> Result<TriremeLedgerClient<Datum, Redeemer>> {
        let ledger = match self.ledger_source {
            LedgerSource::BlockFrost => {
                todo!()
            }
        };
        let keys = match self.key_source {
            KeySource::RawSecretPhrase { phrase_file } => {
                RawSecretPhraseKeys::new(phrase_file, network)
            }
        };
        let inner_client = InnerClient::CML(CMLLedgerCLient::new(ledger, keys, network));
        let trireme_client = TriremeLedgerClient {
            _datum: Default::default(),
            _redeemer: Default::default(),
            inner_client,
        };
        Ok(trireme_client)
    }
}

enum InnerClient<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop> {
    CML(CMLLedgerCLient<BlockFrostLedger, RawSecretPhraseKeys, Datum, Redeemer>),
}

pub struct TriremeLedgerClient<Datum: PlutusDataInterop, Redeemer: PlutusDataInterop> {
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
    inner_client: InnerClient<Datum, Redeemer>,
}

#[async_trait]
impl<Datum: PlutusDataInterop + Send + Sync, Redeemer: PlutusDataInterop + Send + Sync>
    LedgerClient<Datum, Redeemer> for TriremeLedgerClient<Datum, Redeemer>
{
    async fn signer(&self) -> LedgerClientResult<Address> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.signer(),
        }
        .await
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.outputs_at_address(address),
        }
        .await
    }

    async fn issue(&self, tx: UnbuiltTransaction<Datum, Redeemer>) -> LedgerClientResult<TxId> {
        match &self.inner_client {
            InnerClient::CML(cml_client) => cml_client.issue(tx),
        }
        .await
    }
}
