use anyhow::{bail, Result};

use polymesh_api::client::{PairSigner, Signer};
use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_keyring::sr25519::sr25519::Pair;

use crate::util;

/// Generate a Polymesh public address using a 32-byte private key given as a
/// hexadecimal string
pub fn private_key_to_ss58check(priv_key: &str, mainnet: bool) -> Result<String> {
  let signer = util::pairsigner_from_str(priv_key)?;
  let addr = match mainnet {
    true => signer
      .account()
      .to_ss58check_with_version(Ss58AddressFormatRegistry::PolymeshAccount.into()),
    false => signer.account().to_ss58check(),
  };
  Ok(addr)
}

pub fn mnemonic_to_ss58check(
  mnemonic: &str,
  mainnet: bool,
  password: Option<&str>,
) -> Result<String> {
  let pair = match <Pair as sp_core::Pair>::from_phrase(mnemonic, password) {
    Ok((pair, _)) => pair,
    Err(_) => bail!(format!("Invalid 12-word BIP39 mnemonic: {mnemonic}")),
  };
  let signer = PairSigner::new(pair);
  let addr = match mainnet {
    true => signer
      .account()
      .to_ss58check_with_version(Ss58AddressFormatRegistry::PolymeshAccount.into()),
    false => signer.account().to_ss58check(),
  };
  Ok(addr)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_gets_the_correct_address() {
    let signing_addr = "6282c8c97534f8570573ccd4136539b2be1db1dc5b35e224c4db2b51d29c653e";
    let mainnet = false;
    let addr = private_key_to_ss58check(signing_addr, mainnet).unwrap();
    let expected = String::from("5FPAYmXzQhLvFQggnYGNAgrkrUB3GCSoWAfT3NS2ageeGqtt");
    assert_eq!(addr, expected);
  }
}
