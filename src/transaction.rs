use crate::util;
use anyhow::Result;
use polymesh_api::{
  client::{sp_core::crypto::Ss58Codec, AccountId},
  Api,
};

/// Create a transaction. Transaction is then signed with a private key and submitted on-chain. The
/// input dest should be an Ss58-encoded &str, e.g. "5EEiPC3dQ6dvYHQmovFzvpLbsMzCCoCax2oekPBVyq84bWG4"
pub async fn withdraw(
  signing_key: &str,
  dest: &str, // An SS58-encoded adress
  amount: u128,
  mainnet: bool,
) -> Result<String> {
  let dest = AccountId::from_ss58check(dest)?;
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .balances()
    .transfer(dest.into(), amount)?;
  let mut signer = util::pairsigner_from_private_key(signing_key)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

/// Create a transaction. Transaction is then signed with a private key and submitted on-chain.
pub async fn withdraw_with_mnemonic(
  mnemonic: &str,
  dest: &str, // An SS58-encoded adress
  amount: u128,
  mainnet: bool,
) -> Result<String> {
  // The input dest should be an SS58-encoded string, e.g.
  // "5EEiPC3dQ6dvYHQmovFzvpLbsMzCCoCax2oekPBVyq84bWG4"
  let dest = AccountId::from_ss58check(dest)?;
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .balances()
    .transfer(dest.into(), amount)?;
  let mut signer = util::pairsigner_from_mnemonic(mnemonic, None)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}
