use anyhow::Result;

use polymesh_api::client::AccountId;
use polymesh_api::Api;
use sp_core::crypto::Ss58Codec;

use crate::util;

pub async fn free(addr: &str, mainnet: bool) -> Result<u128> {
  let account = AccountId::from_string(addr)?;
  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let account_info = api.query().system().account(account).await?;
  Ok(account_info.data.free)
}

pub async fn staked(addr: &str, mainnet: bool) -> Result<u128> {
  let account = AccountId::from_string(addr)?;
  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let account_info = api.query().system().account(account).await?;
  Ok(account_info.data.reserved)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn it_gets_free_balance() {
    let addr = "5Dext4xTrU8joa6LnPhPQgs6TJH1Jgydr1n2PUyRsBVzTx1A";
    let mainnet = false;
    let res = free(addr, mainnet).await;
    assert!(res.is_ok());
  }

  #[tokio::test]
  async fn it_gets_staked_balance() {
    let addr = "5Dext4xTrU8joa6LnPhPQgs6TJH1Jgydr1n2PUyRsBVzTx1A";
    let mainnet = false;
    let res = staked(addr, mainnet).await;
    assert!(res.is_ok());
  }
}
