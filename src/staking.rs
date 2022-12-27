#![allow(dead_code)]
use anyhow::{bail, Result};

use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_keyring::sr25519::sr25519::Pair;

use polymesh_api::client::{AccountId, MultiAddress, PairSigner};
use polymesh_api::types::pallet_staking::RewardDestination;
use polymesh_api::Api;

use crate::util;

/// Declare to nominate `targets` for the origin controller.
/// Effects will only be felt at the beginning of the next era. This can only be called when
/// [`EraElectionStatus`] is `Closed`.
/// The dispatch origin for this call must be signed by the *controller*, not the stash.
pub async fn nominate(controller_key: &str, operators: Vec<&str>, mainnet: bool) -> Result<String> {
  let signing_key: [u8; 32] =
    match hex::decode(controller_key.strip_prefix("0x").unwrap_or(controller_key)) {
      Ok(v) => v.as_slice().try_into()?,
      Err(_) => bail!("expected 32-byte hexadecimal signing key, got {controller_key}"),
    };
  let pair = <Pair as sp_core::Pair>::from_seed(&signing_key);
  let mut signer = PairSigner::new(pair);

  let account_ids: Result<Vec<_>, _> = operators
    .iter()
    .map(|&ss58| AccountId::from_string(ss58))
    .collect();
  if account_ids.is_err() {
    bail!("expected operator nodes as SS58-formatted addresses, got {operators:?}")
  }
  let targets: Vec<MultiAddress<AccountId, u32>> = account_ids
    .expect("successful mapping of input to Account IDs should have been ensured")
    .iter()
    .map(|&id| MultiAddress::from(id))
    .collect();

  let api = Api::new(util::url(mainnet)).await?;
  let call = api.call().staking().nominate(targets)?;
  util::sign_submit_and_watch(&api, &call, &mut signer).await
}

/// Take the origin account as a stash and lock up `value` of its balance.
/// `controller` will be the account that controls it.
pub async fn bond(
  stash_key: &str,
  controller_addr: &str,
  value: u128,
  mainnet: bool,
) -> Result<String> {
  let stash_key: [u8; 32] = hex::decode(stash_key.strip_prefix("0x").unwrap_or(stash_key))?
    .as_slice()
    .try_into()?;
  let pair = <Pair as sp_core::Pair>::from_seed(&stash_key);
  let mut signer = PairSigner::new(pair);
  let controller = MultiAddress::from(AccountId::from_string(controller_addr)?);

  let api = Api::new(util::url(mainnet)).await?;
  let call = api
    .call()
    .staking()
    .bond(controller, value, RewardDestination::Stash)?;
  util::sign_submit_and_watch(&api, &call, &mut signer).await
}

/// The AccountIds (public) of validator nodes
pub async fn validators(mainnet: bool) -> Result<Vec<String>> {
  let api = Api::new(util::url(mainnet)).await?;
  let account_ids = api.query().session().validators().await?;
  let operators = match mainnet {
    true => account_ids
      .iter()
      .map(|id| id.to_ss58check_with_version(Ss58AddressFormatRegistry::PolymeshAccount.into()))
      .collect(),
    false => account_ids.iter().map(|id| id.to_ss58check()).collect(),
  };
  Ok(operators)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn it_gets_validators_on_testnet() {
    let mainnet = false;
    let res = validators(mainnet).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn it_gets_validators_on_mainnet() {
    let mainnet = true;
    let res = validators(mainnet).await;
    assert!(res.is_ok());
  }
}
