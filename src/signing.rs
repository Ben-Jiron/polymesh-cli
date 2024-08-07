#![allow(unused_imports, unused_variables)]
use crate::util;
use anyhow::Result;
use parity_scale_codec::Encode;
use polymesh_api::client::{
  sp_core::crypto::Ss58Codec,
  sp_core::sr25519::{Public, Signature},
  Signer,
};

/// Using a private key (given as a 32-byte hexadecimal string without a `0x` prefix)
/// and a payload (given as a hexadecimal string without a `0x` prefix), this function
/// uses the Polymesh API to sign the payload, yielding a signature which can be
/// validated against the user's public address (a base-64 encoded address starting with 5).
pub async fn sign_payload(signing_key: &str, payload: &str) -> Result<String> {
  let payload = hex::decode(payload.strip_prefix("0x").unwrap_or(payload))?;
  let signer = util::pairsigner_from_private_key(signing_key)?;
  let res = signer.sign(&payload).await?;
  Ok(hex::encode(res.encode()))
}

/// Verify a signature against a payload and its signer's public address
pub fn verify_signature(signature: &str, ss58_addr: &str, payload: &str) -> bool {
  let signature =
    hex::decode(signature.strip_prefix("0x").unwrap_or(signature)).unwrap_or_default();
  let payload = hex::decode(payload.strip_prefix("0x").unwrap_or(payload)).unwrap_or_default();
  todo!()
  // match (
  //   Signature::from_slice(&signature),
  //   Public::from_ss58check(ss58_addr),
  // ) {
  //   (Some(sig), Ok(public)) => public. sig.verify(&payload[..], &public),
  //   _ => false,
  // }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn it_signs_a_payload() {
    let signing_addr = "6282c8c97534f8570573ccd4136539b2be1db1dc5b35e224c4db2b51d29c653e";
    let payload = "deadbeef";
    let signature = sign_payload(signing_addr, payload).await;
    assert!(signature.is_ok());
  }

  #[tokio::test]
  async fn it_verifies_a_signature() {
    let signing_addr = "6282c8c97534f8570573ccd4136539b2be1db1dc5b35e224c4db2b51d29c653e";
    let ss58_addr = "5FPAYmXzQhLvFQggnYGNAgrkrUB3GCSoWAfT3NS2ageeGqtt";
    let payload = "deadbeef";
    let signature = sign_payload(signing_addr, payload).await.unwrap();
    let is_valid = verify_signature(&signature, ss58_addr, payload);
    assert!(is_valid);
  }

  #[tokio::test]
  async fn it_fails_to_sign_if_signing_addr_is_too_short() {
    let signing_addr = "deadbeef";
    let payload = "deadbeef";
    let signature = sign_payload(signing_addr, payload).await;
    assert!(signature.is_err());
  }

  #[tokio::test]
  async fn it_fails_to_sign_if_payload_has_odd_length() {
    let signing_addr = "6282c8c97534f8570573ccd4136539b2be1db1dc5b35e224c4db2b51d29c653e";
    let payload = "deadbeef4";
    let signature = sign_payload(signing_addr, payload).await;
    assert!(signature.is_err());
  }
}
