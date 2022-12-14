use clap::{arg, value_parser, Command};

pub fn command() -> Command {
  Command::new("polymesh-cli")
    .subcommand(
      Command::new("send")
        .about("Send POLYX between accounts.")
        .args(&[
          arg!(key: -k --key <KEY> "32-byte hexadecimal private key of signing account")
            .required(true),
          arg!(amount: -a --amount <AMOUNT> "Amount to transfer in POLYX")
            .value_parser(value_parser!(f64))
            .required(true),
          arg!(destination: -d --destination <DESTINATION> "Public SS58 address of receiving account")
            .required(true),
          arg!(mainnet: --mainnet "If set, performs transaction on mainnet").required(false),
        ])
    )
    .subcommand(
      Command::new("sign")
        .about("Sign a payload of bytes with Polymesh private key.")
        .args(&[
          arg!(key: -k --key <KEY> "32-byte hexadecimal private key of signing account")
            .required(true),
          arg!(payload: -p --payload <PAYLOAD> "Payload of bytes given as a hexadecimal string")
            .required(true),
        ])
    )
    .subcommand(
      Command::new("verify")
        .about("Verify a signature against a user's public key and the unsigned payload.")
        .args(&[
          arg!(address: -a --address <ADDRESS> "SS58-formatted public address of signer")
            .required(true),
          arg!(payload: -p --payload <PAYLOAD> "Payload of bytes given as a hexadecimal string")
            .required(true),
          arg!(signature: -s --signature <SIGNATURE> "Signature bytes given as a hexadecimal string")
            .required(true),
        ])
    )
    .subcommand(
      Command::new("address")
        .about("Get a user's public address given their hexadecimal private key")
        .args(&[
          arg!(key: "32-byte hexadecimal private key of signing account").required(true),
          arg!(mainnet: --mainnet "If set, returns mainnet address (starting with 2)").required(false),
        ])
    )
    .subcommand(
      Command::new("secondary")
        .about("Add and remove secondary keys from primary signing account.")
        .subcommand(
          Command::new("add")
            .short_flag('a')
            .args(&[
              arg!(mnemonic: -m --mnemonic <MNEMONIC> "BIP39 secret mnemonic phrase for primary account")
                .required(true),
              arg!(secondary_key: -s --secondary <SECONDARY> "32-byte hexadecimal private signing key of secondary")
                .alias("who")
                .short_alias('w')
                .required(true),
              arg!(expires_after: -e --expires <EXPIRY> "Set the duration (in seconds) for which secondary will have authorization")
                .value_parser(value_parser!(u64))
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
        .subcommand(
          Command::new("remove")
            .short_flag('r')
            .args(&[
              arg!(mnemonic: -m --mnemonic <MNEMONIC> "BIP39 secret mnemonic phrase for primary account")
                .required(true),
              arg!(who: -w --who <ADDRESS> "SS58-formatted public address of secondary key")
                .alias("secondary")
                .short_alias('s')
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
    )
}
