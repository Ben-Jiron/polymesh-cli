use anyhow::{bail, Result};

use polymesh_api::client::{AccountId, PairSigner};
use polymesh_api::Api;
use sp_core::crypto::Ss58Codec;
use sp_keyring::sr25519::sr25519::Pair;

use crate::util;

/// Create a transaction. Transaction is then signed with a private key and submitted on-chain.
pub async fn withdraw(
  signing_key: &str,
  dest: &str, // An SS58-encoded adress
  amount: u128,
  mainnet: bool,
) -> Result<String> {
  // Signing key is a 32-byte hexadecimal private key
  let signing_key: [u8; 32] = hex::decode(signing_key.strip_prefix("0x").unwrap_or(signing_key))?
    .as_slice()
    .try_into()?;
  // The input dest should be an Ss58-encoded &str, e.g.
  // "5EEiPC3dQ6dvYHQmovFzvpLbsMzCCoCax2oekPBVyq84bWG4"
  let dest = AccountId::from_ss58check(dest)?;
  let pair = <Pair as sp_core::Pair>::from_seed(&signing_key);
  let mut signer = PairSigner::new(pair);

  // RPC URL
  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let call = api.call().balances().transfer(dest.into(), amount)?;

  util::sign_submit_and_watch(&api, &call, &mut signer).await
}

/// Create a transaction. Transaction is then signed with a private key and submitted on-chain.
pub async fn withdraw_with_mnemonic(
  mnemonic: &str,
  dest: &str, // An SS58-encoded adress
  amount: u128,
  mainnet: bool,
) -> Result<String> {
  // The input dest should be an Ss58-encoded &str, e.g.
  // "5EEiPC3dQ6dvYHQmovFzvpLbsMzCCoCax2oekPBVyq84bWG4"
  let dest = AccountId::from_ss58check(dest)?;
  let pair = match <Pair as sp_core::Pair>::from_string(mnemonic, None) {
    Ok(p) => p,
    Err(_) => bail!("failed to convert mnemonic phrase to SR25519 keypair"),
  };
  let mut signer = PairSigner::new(pair);

  // RPC URL
  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let call = api.call().balances().transfer(dest.into(), amount)?;

  util::sign_submit_and_watch(&api, &call, &mut signer).await
}
