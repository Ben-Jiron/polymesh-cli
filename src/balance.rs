use crate::util;
use anyhow::Result;
use polymesh_api::{
  client::{sp_core::crypto::Ss58Codec, AccountId},
  Api,
};

pub async fn free(addr: &str, mainnet: bool) -> Result<u128> {
  let account_info = Api::new(util::url(mainnet))
    .await?
    .query()
    .system()
    .account(AccountId::from_string(addr)?)
    .await?;
  Ok(account_info.data.free)
}

pub async fn staked(addr: &str, mainnet: bool) -> Result<u128> {
  let account_info = Api::new(util::url(mainnet))
    .await?
    .query()
    .system()
    .account(AccountId::from_string(addr)?)
    .await?;
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
