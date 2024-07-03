use crate::util;
use anyhow::Result;
use polymesh_api::client::{
  sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec},
  Signer,
};

fn signer_ss58check(signer: &impl Signer, mainnet: bool) -> String {
  if mainnet {
    signer
      .account()
      .to_ss58check_with_version(Ss58AddressFormatRegistry::PolymeshAccount.into())
  } else {
    signer.account().to_ss58check()
  }
}

/// Generate a Polymesh public address using a 32-byte private key given as a hexadecimal string
pub fn private_key_to_ss58check(priv_key: &str, mainnet: bool) -> Result<String> {
  let signer = util::pairsigner_from_private_key(priv_key)?;
  Ok(signer_ss58check(&signer, mainnet))
}

pub fn mnemonic_to_ss58check(
  mnemonic: &str,
  mainnet: bool,
  password_override: Option<&str>,
) -> Result<String> {
  let signer = util::pairsigner_from_mnemonic(mnemonic, password_override)?;
  Ok(signer_ss58check(&signer, mainnet))
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
