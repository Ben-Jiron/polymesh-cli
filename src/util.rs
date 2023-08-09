use anyhow::{bail, Context, Result};

use parity_scale_codec::Encode;
use polymesh_api::client::{Era, Extra, ExtrinsicV4, PairSigner, SignedPayload, Signer};
use polymesh_api::{Api, ChainApi, WrappedCall};
use sp_keyring::sr25519::sr25519::Pair;

const MAINNET_URL: &str = "wss://mainnet-rpc.polymesh.network";
const TESTNET_URL: &str = "wss://testnet-rpc.polymesh.live";

/// Get RPC URL for mainnet or testnet
pub fn url(mainnet: bool) -> &'static str {
  match mainnet {
    true => MAINNET_URL,
    false => TESTNET_URL,
  }
}

/// Decodes a 32-byte hexadecimal string into a byte slice to be used as a key/seed.
pub fn decode_hex_key(key: &str) -> Result<[u8; 32]> {
  hex::decode(key.strip_prefix("0x").unwrap_or(key))
    .context("invalid hex: key needs to be a 32-byte hexadecimal string")?
    .as_slice()
    .try_into()
    .context("invalid length: key needs to be a 32-byte hexadecimal string")
}

pub fn pairsigner_from_str(key: &str) -> Result<PairSigner<Pair>> {
  let key = decode_hex_key(key)?;
  let pair = <Pair as sp_core::Pair>::from_seed(&key);
  Ok(PairSigner::new(pair))
}

pub fn pairsigner_from_mnemonic(
  mnemonic: &str,
  password: Option<&str>,
) -> Result<PairSigner<Pair>> {
  let pair = match <Pair as sp_core::Pair>::from_phrase(mnemonic, password) {
    Ok((pair, _)) => pair,
    Err(_) => bail!("failed to generate keypair from given mnemonic"),
  };
  Ok(PairSigner::new(pair))
}

/// Taking in a `WrappedCall`, encode and sign the call, then submit to the chain.
pub async fn sign_submit_and_watch(
  api: &Api,
  call: &WrappedCall<'_>,
  signer: &mut impl Signer,
) -> Result<String> {
  // https://docs.rs/polymesh-api-client/0.2.0/src/polymesh_api_client/transaction.rs.html#256-283
  let account = signer.account();
  let client = <Api as ChainApi>::client(api);
  let nonce = match signer.nonce() {
    Some(0) | None => api.get_nonce(account).await?,
    Some(nonce) => nonce,
  };

  let encoded_call = call.encoded();
  let extra = Extra::new(Era::Immortal, nonce);
  let payload = SignedPayload::new(&encoded_call, &extra, client.get_signed_extra()).encode();

  let sig = signer.sign(&payload[..]).await?;
  let xt = ExtrinsicV4::signed(account, sig, extra, encoded_call);
  let (_tx_hex, tx_hash) = xt.as_hex_and_hash();
  call.submit_and_watch(xt).await?;
  // Update nonce if call was submitted
  signer.set_nonce(nonce + 1);

  Ok(String::from("0x") + &hex::encode(tx_hash.as_bytes()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn it_gets_api_on_mainnet() {
    let mainnet = true;
    let url = url(mainnet);
    let api = Api::new(url).await;
    assert!(api.is_ok());
  }

  #[tokio::test]
  async fn it_gets_api_on_testnet() {
    let mainnet = false;
    let url = url(mainnet);
    let api = Api::new(url).await;
    assert!(api.is_ok());
  }
}
