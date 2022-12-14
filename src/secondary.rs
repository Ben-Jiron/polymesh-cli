use anyhow::{bail, Result};

use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::Ss58Codec;
use sp_keyring::sr25519::sr25519::Pair;

use polymesh_api::client::{AccountId, IdentityId, PairSigner, Signer};
use polymesh_api::types::{
  polymesh_common_utilities::traits::identity::SecondaryKeyWithAuth,
  polymesh_primitives::{
    secondary_key::{KeyRecord, Permissions, SecondaryKey},
    subset::SubsetRestriction,
  },
  primitive_types::H512,
};
use polymesh_api::Api;

use std::time::{Duration, SystemTime};

use crate::util;

pub type Moment = u64;
pub type AuthorizationNonce = u64;

/// TargetIdAuthorization for adding a secondary key to a DID
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
struct TargetIdAuthorization {
  /// Target identity which is authorized to make an operation.
  pub target_id: IdentityId,
  /// It HAS TO be `target_id` authorization nonce: See `Identity::offchain_authorization_nonce`
  pub nonce: AuthorizationNonce,
  pub expires_at: Moment,
}

#[allow(dead_code)]
pub async fn add_secondary_auth(api: &Api, did: &[u8; 32], expires_after: u64) -> Result<Vec<u8>> {
  let target_id = IdentityId(*did);
  let identity_query = api.query().identity();
  let nonce = identity_query
    .off_chain_authorization_nonce(target_id)
    .await?;
  let expires_at = SystemTime::now()
    .checked_add(Duration::from_secs(expires_after))
    .unwrap_or_else(SystemTime::now)
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("logic error in getting Unix time")
    .as_secs();
  let auth = TargetIdAuthorization {
    target_id,
    nonce,
    expires_at,
  };
  Ok(auth.encode())
}

pub async fn add(
  primary_mnemonic: &str, // private key of primary
  secondary_key: &str,    // private key of secondary
  expires_after: u64,     // authorization after this many seconds
  mainnet: bool,          // On mainnet (as opposed to testnet)?
) -> Result<String> {
  let secondary_key: [u8; 32] =
    hex::decode(secondary_key.strip_prefix("0x").unwrap_or(secondary_key))?
      .as_slice()
      .try_into()?;
  // Get PairSigners for primary and secondary keys
  let primary_pair = match <Pair as sp_core::Pair>::from_string(primary_mnemonic, None) {
    Ok(pair) => pair,
    Err(_) => bail!("failed to convert mnemonic to SR25519 keypair"),
  };
  let mut primary_signer = PairSigner::new(primary_pair);
  let secondary_pair = <Pair as sp_core::Pair>::from_seed(&secondary_key);
  let secondary_signer = PairSigner::new(secondary_pair);

  // Create an API instance with the desired node RPC URL
  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let identity_query = api.query().identity();

  // Create TargetIdAuthorization from target DID, the DID's nonce, and an expiry
  let target_id = match identity_query.key_records(primary_signer.account).await? {
    Some(KeyRecord::PrimaryKey(did)) => did,
    Some(_) => bail!("must use primary key to add secondary keys"),
    None => bail!("{:?} doesn't have an identity", primary_signer.account),
  };
  let nonce = identity_query
    .off_chain_authorization_nonce(target_id)
    .await
    .unwrap();
  let expires_at = SystemTime::now()
    .checked_add(Duration::from_secs(expires_after))
    .unwrap_or_else(SystemTime::now)
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("logic error in getting Unix time")
    .as_millis() as u64;
  let auth = TargetIdAuthorization {
    target_id,
    nonce,
    expires_at,
  };
  let auth_data = auth.encode();
  // After signing, the signature always comes back as 65 bytes.
  let secondary_signature_65_bytes = secondary_signer.sign(&auth_data).await?.encode();
  let secondary_signature: [u8; 64] = secondary_signature_65_bytes[1..].try_into()?;
  let auth_signature = H512(secondary_signature);

  // Create a SecondaryKeyWithAuth to be submitted on-chain
  // https://docs.rs/polymesh-api/0.3.3/polymesh_api/polymesh/types/polymesh_primitives/subset/enum.SubsetRestriction.html
  let permissions = Permissions {
    asset: SubsetRestriction::Whole,
    extrinsic: SubsetRestriction::Whole,
    portfolio: SubsetRestriction::Whole,
  };
  let secondary_key = SecondaryKey {
    key: secondary_signer.account(),
    permissions,
  };
  let secondary_key_with_auth = SecondaryKeyWithAuth {
    secondary_key,
    auth_signature,
  };

  // Use the API to create a WrappedCall to be submitted with the primary account
  let additional_keys = vec![secondary_key_with_auth];
  let call = api
    .call()
    .identity()
    .add_secondary_keys_with_authorization(additional_keys, expires_at)?;
  util::sign_submit_and_watch(&api, &call, &mut primary_signer).await
}

/// Removes secondary key from account
pub async fn remove(primary_mnemonic: &str, who: &str, mainnet: bool) -> Result<String> {
  let who = AccountId::from_ss58check(who)?;
  let pair = match <Pair as sp_core::Pair>::from_string(primary_mnemonic, None) {
    Ok(pair) => pair,
    Err(_) => bail!("failed to convert mnemonic to SR25519 keypair"),
  };
  let mut signer = PairSigner::new(pair);

  let url = util::url(mainnet);
  let api = Api::new(url).await?;
  let call = api.call().identity().remove_secondary_keys(vec![who])?;

  util::sign_submit_and_watch(&api, &call, &mut signer).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn it_creates_auth_data() {
    let url = util::url(false); // testnet
    let api = Api::new(url).await.unwrap();
    let did_hex = "0x7b5f2e3da9b75966d65ee9e4dfcda6db539d8697269c5644145892ae61e9d39f";
    let did: [u8; 32] = hex::decode(did_hex.strip_prefix("0x").unwrap_or(did_hex))
      .unwrap()
      .as_slice()
      .try_into()
      .unwrap();
    let expires_after = 3600;
    let res = add_secondary_auth(&api, &did, expires_after).await;
    assert!(res.is_ok());
    let encoded = res.unwrap();
    println!("encoded length: {}", encoded.len());
  }
}
