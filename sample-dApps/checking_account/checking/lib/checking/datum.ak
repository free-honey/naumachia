use aiken/transaction/value.{PolicyId}

pub type CheckingAccountDatum {
  owner: ByteArray,
  spend_token_policy: PolicyId,
}