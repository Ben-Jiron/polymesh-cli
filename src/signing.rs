use anyhow::{bail, Result};

use parity_scale_codec::Encode;
use sp_core::crypto::Ss58Codec;
use sp_keyring::sr25519::sr25519::{Pair, Public, Signature};
use sp_runtime::{traits::Verify, MultiSignature};

use polymesh_api::client::{PairSigner, Signer};

/// Using a private key (given as a 32-byte hexadecimal string without a `0x` prefix)
/// and a payload (given as a hexadecimal string without a `0x` prefix), this function
/// uses the Polymesh API to sign the payload, yielding a signature which can be
/// validated against the user's public address (a base-64 encoded address starting with 5).
pub async fn sign_payload(signing_addr: &str, payload: &str) -> Result<String> {
  let payload = hex::decode(payload.strip_prefix("0x").unwrap_or(payload))?;
  let signing_addr: [u8; 32] =
    hex::decode(signing_addr.strip_prefix("0x").unwrap_or(signing_addr))?
      .as_slice()
      .try_into()?;
  let pair = <Pair as sp_core::Pair>::from_seed(&signing_addr);
  let signer = PairSigner::new(pair);

  match signer.sign(&payload).await? {
    MultiSignature::Sr25519(sig) => Ok(hex::encode(sig.encode())),
    _ => bail!("only SR25519 signatures supported"),
  }
}

/// Verify a signature against a payload and its signer's public address
pub fn verify_signature(signature: &str, ss58_addr: &str, payload: &str) -> bool {
  let signature =
    hex::decode(signature.strip_prefix("0x").unwrap_or(signature)).unwrap_or_default();
  let payload = hex::decode(payload.strip_prefix("0x").unwrap_or(payload)).unwrap_or_default();
  match (
    Signature::from_slice(&signature),
    Public::from_ss58check(ss58_addr),
  ) {
    (Some(sig), Ok(public)) => sig.verify(&payload[..], &public),
    _ => false,
  }
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
