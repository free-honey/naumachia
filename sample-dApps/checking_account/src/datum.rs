use crate::Address;
use naumachia::scripts::context::PubKeyHash;
use naumachia::scripts::raw_validator_script::plutus_data::{Constr, PlutusData};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CheckingAccountDatums {
    CheckingAccount(CheckingAccount),
    AllowedPuller(AllowedPuller),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CheckingAccount {
    pub owner: PubKeyHash,
    pub spend_token_policy: Vec<u8>,
}

impl From<CheckingAccount> for CheckingAccountDatums {
    fn from(value: CheckingAccount) -> Self {
        CheckingAccountDatums::CheckingAccount(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AllowedPuller {
    pub owner: PubKeyHash,
    pub puller: PubKeyHash,
    pub amount_lovelace: u64,
    pub next_pull: i64,
    pub period: i64,
    pub spending_token: Vec<u8>,
    pub checking_account_address: Address,
    pub checking_account_nft: Vec<u8>,
}

impl From<AllowedPuller> for CheckingAccountDatums {
    fn from(value: AllowedPuller) -> Self {
        CheckingAccountDatums::AllowedPuller(value)
    }
}

impl From<CheckingAccountDatums> for PlutusData {
    fn from(value: CheckingAccountDatums) -> Self {
        match value {
            CheckingAccountDatums::CheckingAccount(CheckingAccount {
                owner,
                spend_token_policy,
            }) => {
                let owner_data = owner.into();
                let policy_data = PlutusData::BoundedBytes(spend_token_policy);
                PlutusData::Constr(Constr {
                    constr: 0,
                    fields: vec![owner_data, policy_data],
                })
            }
            CheckingAccountDatums::AllowedPuller(AllowedPuller {
                owner,
                puller,
                amount_lovelace,
                next_pull,
                period,
                spending_token,
                checking_account_address,
                checking_account_nft,
            }) => {
                let owner = owner.into();
                let puller = puller.into();
                let amount_lovelace = PlutusData::BigInt((amount_lovelace as i64).into());
                let next_pull = PlutusData::BigInt(next_pull.into());
                let period = PlutusData::BigInt(period.into());
                let spending_token = PlutusData::BoundedBytes(spending_token);
                let checking_account_nft = PlutusData::BoundedBytes(checking_account_nft);
                PlutusData::Constr(Constr {
                    constr: 0,
                    fields: vec![
                        owner,
                        puller,
                        amount_lovelace,
                        next_pull,
                        period,
                        spending_token,
                        checking_account_address.into(),
                        checking_account_nft,
                    ],
                })
            }
        }
    }
}

impl TryFrom<PlutusData> for CheckingAccountDatums {
    type Error = ();

    fn try_from(_value: PlutusData) -> Result<Self, Self::Error> {
        todo!()
    }
}
