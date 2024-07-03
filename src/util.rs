use anyhow::Result;
use polymesh_api::{
  client::{
    sp_core::{sr25519, Pair},
    PairSigner, Signer,
  },
  WrappedCall,
};

const MAINNET_URL: &str = "wss://mainnet-rpc.polymesh.network";
const TESTNET_URL: &str = "wss://testnet-rpc.polymesh.live";

/// Get RPC URL for mainnet or testnet
pub fn url(mainnet: bool) -> &'static str {
  if mainnet {
    MAINNET_URL
  } else {
    TESTNET_URL
  }
}

pub fn pairsigner_from_private_key(key: &str) -> Result<PairSigner<sr25519::Pair>> {
  let pair = sr25519::Pair::from_seed_slice(&hex::decode(key)?)?;
  Ok(PairSigner::new(pair))
}

pub fn pairsigner_from_mnemonic(
  mnemonic: &str,
  password_override: Option<&str>,
) -> Result<PairSigner<sr25519::Pair>> {
  Ok(PairSigner::from_string(mnemonic, password_override)?)
}

/// Sign and submit a transaction, returning the hash as a hexadecimal string with an `0x` prefix.
pub async fn sign_submit_and_watch(call: &WrappedCall, signer: &mut impl Signer) -> Result<String> {
  Ok(String::from("0x") + &hex::encode(call.execute(signer).await?.hash()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use polymesh_api::Api;

  #[tokio::test]
  async fn it_gets_api_on_mainnet() {
    let api = Api::new(url(true)).await;
    assert!(api.is_ok());
  }

  #[tokio::test]
  async fn it_gets_api_on_testnet() {
    let api = Api::new(url(false)).await;
    assert!(api.is_ok());
  }
}
