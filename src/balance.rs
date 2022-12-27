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
