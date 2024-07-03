use crate::util;
use anyhow::{bail, Result};
use parity_scale_codec::{Decode, Encode};
use polymesh_api::{
  client::{sp_core::crypto::Ss58Codec, AccountId, IdentityId, Signer},
  types::{
    polymesh_common_utilities::traits::identity::SecondaryKeyWithAuth,
    polymesh_primitives::{
      secondary_key::{KeyRecord, Permissions, SecondaryKey},
      subset::SubsetRestriction,
    },
    primitive_types::H512,
  },
  Api,
};
use std::time::{Duration, SystemTime};

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

pub async fn add_secondary_auth(
  api: &Api,
  primary_account: &AccountId,
  expires_at: u64,
) -> Result<Vec<u8>> {
  let identity_query = api.query().identity();
  // Create TargetIdAuthorization from target DID, the DID's nonce, and an expiry
  let target_id = match identity_query.key_records(*primary_account).await? {
    Some(KeyRecord::PrimaryKey(did)) => did,
    Some(_) => bail!("must use primary key to add secondary keys"),
    None => bail!("{:?} doesn't have an identity", *primary_account),
  };
  let nonce = identity_query
    .off_chain_authorization_nonce(target_id)
    .await?;
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
  // Get PairSigners for primary and secondary keys
  let mut primary_signer = util::pairsigner_from_mnemonic(primary_mnemonic, None)?;
  let secondary_signer = util::pairsigner_from_private_key(secondary_key)?;
  let api = Api::new(util::url(mainnet)).await?;

  // Create TargetIdAuthorization from target DID, the DID's nonce, and an expiry
  let expires_at = SystemTime::now()
    .checked_add(Duration::from_secs(expires_after))
    .unwrap_or_else(SystemTime::now)
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("logic error in getting Unix time")
    .as_millis()
    .min(u64::MAX as u128) as u64;
  let auth_data = add_secondary_auth(&api, &primary_signer.account, expires_at).await?;
  // After signing, the signature always comes back as 65 bytes (ECDSA signature)
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

  let additional_keys = vec![secondary_key_with_auth];
  let call = api
    .call()
    .identity()
    .add_secondary_keys_with_authorization(additional_keys, expires_at)?;
  util::sign_submit_and_watch(&call, &mut primary_signer).await
}

/// Removes secondary key from account
pub async fn remove(primary_mnemonic: &str, who: &str, mainnet: bool) -> Result<String> {
  let who = AccountId::from_ss58check(who)?;
  let call = Api::new(util::url(mainnet))
    .await?
    .call()
    .identity()
    .remove_secondary_keys(vec![who])?;
  let mut signer = util::pairsigner_from_mnemonic(primary_mnemonic, None)?;
  util::sign_submit_and_watch(&call, &mut signer).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  #[ignore]
  async fn it_creates_auth_data() {
    let url = util::url(false); // testnet
    let _api = Api::new(url).await.unwrap();
    // let account_id = AccountId::from_slice(did);
    let _expires_at = SystemTime::now()
      .checked_add(Duration::from_secs(24 * 60 * 60 * 365))
      .unwrap_or_else(SystemTime::now)
      .duration_since(SystemTime::UNIX_EPOCH)
      .expect("logic error in getting Unix time")
      .as_millis()
      .min(u64::MAX as u128) as u64;
    // let res = add_secondary_auth(&api, &did, expires_at).await;
    // assert!(res.is_ok());
    // let encoded = res.unwrap();
    // println!("encoded length: {}", encoded.len());
  }
}
