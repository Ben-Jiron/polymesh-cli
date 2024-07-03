use crate::util;
use anyhow::{Context, Result};
use polymesh_api::{
  client::{
    sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec},
    AccountId, MultiAddress,
  },
  types::pallet_staking::RewardDestination,
  Api,
};

/// Declare to nominate `targets` for the origin controller.
/// Effects will only be felt at the beginning of the next era. This can only be called when
/// [`EraElectionStatus`] is `Closed`.
/// The dispatch origin for this call must be signed by the *controller*, not the stash.
pub async fn nominate(controller_key: &str, operators: Vec<&str>, mainnet: bool) -> Result<String> {
  let account_ids: Result<Vec<_>, _> = operators
    .iter()
    .map(|&ss58| AccountId::from_string(ss58))
    .collect();
  let targets: Vec<MultiAddress<AccountId, u32>> = account_ids
    .context(format!(
      "expected operator nodes as SS58-formatted addresses, got {operators:?}"
    ))?
    .iter()
    .map(|&id| MultiAddress::from(id))
    .collect();
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .staking()
    .nominate(targets)?;
  let mut signer = util::pairsigner_from_private_key(controller_key)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

/// Take the origin account as a stash and lock up `value` of its balance.
/// `controller` will be the account that controls it.
pub async fn bond(
  stash_key: &str,
  controller_addr: &str,
  value: u128,
  mainnet: bool,
) -> Result<String> {
  let call = Api::new(util::url(mainnet)).await?.call().staking().bond(
    MultiAddress::from(AccountId::from_string(controller_addr)?),
    value,
    RewardDestination::Stash,
  )?;
  let mut signer = util::pairsigner_from_private_key(stash_key)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

/// As a controller, unbond `value` micro-POLYX from being staked by stash.
#[allow(dead_code)]
pub async fn unbond(controller_key: &str, value: u128, mainnet: bool) -> Result<String> {
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .staking()
    .unbond(value)?;
  let mut signer = util::pairsigner_from_private_key(controller_key)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

pub async fn bond_extra(stash_key: &str, amount: u128, mainnet: bool) -> Result<String> {
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .staking()
    .bond_extra(amount)?;
  let mut signer = util::pairsigner_from_private_key(stash_key)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

#[allow(dead_code)]
pub async fn bond_extra_with_mnemonic(
  mnemonic: &str,
  amount: u128,
  mainnet: bool,
) -> Result<String> {
  let api = Api::new(util::url(mainnet)).await?;
  let call = api.call().staking().bond_extra(amount)?;
  let mut signer = util::pairsigner_from_mnemonic(mnemonic, None)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

/// Withdraw unbonded tokens when [EraElectionStatus] is `Closed`.
pub async fn withdraw_unbonded(controller_key: &str, mainnet: bool) -> Result<String> {
  let api = Api::new(util::url(mainnet)).await?;
  let mut signer = util::pairsigner_from_private_key(controller_key)?;
  let ledger = api
    .query()
    .staking()
    .ledger(signer.account)
    .await?
    .context("no ledger found")?;
  // Get slashing spans of stash account
  let num_slashing_spans = if ledger.active > 0 {
    0
  } else {
    match api.query().staking().slashing_spans(ledger.stash).await? {
      None => 0,                                     // no slashing spans
      Some(spans) => (spans.prior.len() + 1) as u32, // number of prior spans + last span
    }
  };

  let call = api.call().staking().withdraw_unbonded(num_slashing_spans)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

#[allow(dead_code)]
pub async fn active_in_ledger(controller_key: &str, mainnet: bool) -> Result<u128> {
  let ledger = Api::new(util::url(mainnet))
    .await?
    .query()
    .staking()
    .ledger(util::pairsigner_from_private_key(controller_key)?.account)
    .await?
    .context("no ledger found")?;
  Ok(ledger.active)
}

/// Get sum of all staking rewards
#[allow(dead_code)]
pub async fn total_rewarded(controller_key: &str, mainnet: bool) -> Result<u32> {
  let ledger = Api::new(util::url(mainnet))
    .await?
    .query()
    .staking()
    .ledger(util::pairsigner_from_private_key(controller_key)?.account)
    .await?
    .context("not a controller")?;
  Ok(ledger.claimed_rewards.iter().sum())
}

/// The AccountIds (public) of validator nodes
pub async fn validators(mainnet: bool) -> Result<Vec<String>> {
  let account_ids = Api::new(util::url(mainnet))
    .await?
    .query()
    .session()
    .validators()
    .await?;
  let operators = if mainnet {
    account_ids
      .iter()
      .map(|id| id.to_ss58check_with_version(Ss58AddressFormatRegistry::PolymeshAccount.into()))
      .collect()
  } else {
    account_ids.iter().map(|id| id.to_ss58check()).collect()
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

  #[tokio::test]
  #[ignore]
  async fn it_unbonds() {
    let mainnet = false;
    let value = 80 * 1_000_000;
    let controller_key = "9a62194397c8ccd1a8b4546afa594937e75f469381739829f979ce459910a584";
    let res = unbond(controller_key, value, mainnet).await;
    assert!(res.is_ok());
    println!("{}", res.unwrap());
    assert!(false);
  }

  #[tokio::test]
  #[ignore]
  async fn it_withdraws_unbonded() {
    let mainnet = false;
    // let controller_key = "9173628750a527f9cdaa69ecbec47b11981299c4e47307b2d7df75a8b0f7d01f";
    // let controller_key = "9a62194397c8ccd1a8b4546afa594937e75f469381739829f979ce459910a584";
    // gnarwhal
    let controller_key = "88a3c978f0ebcda75605516e8c7bdc1a437fff484c1a4c24a663f7149e1271e2";
    let res = withdraw_unbonded(controller_key, mainnet).await;
    assert!(res.is_ok());
    println!("Result: {}", res.unwrap());
    assert!(false);
  }

  #[tokio::test]
  async fn it_gets_sum_of_rewards() {
    let mainnet = false;
    // not a controller (all unbonded)
    // let controller_key = "88a3c978f0ebcda75605516e8c7bdc1a437fff484c1a4c24a663f7149e1271e2"; //
    let controller_key = "9173628750a527f9cdaa69ecbec47b11981299c4e47307b2d7df75a8b0f7d01f";
    let res = total_rewarded(controller_key, mainnet).await;
    assert!(res.is_ok());
    let rewards = res.unwrap();
    println!("Rewards: {} POLYX", rewards as f64 * 1e-6);
  }
}
