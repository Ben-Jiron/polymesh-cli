use clap::{arg, value_parser, Command};

pub fn command() -> Command {
  Command::new("polymesh-cli")
    .about("Utilities for interacting with the Polymesh blockchain")
    .subcommand_required(true)

    // Subcommand: send
    .subcommand(
      Command::new("send")
        .about("Send POLYX between accounts.")
        .args(&[
          arg!(key: -k --key <KEY> "32-byte hexadecimal private key of signing account")
            .required_unless_present("mnemonic"),
          arg!(mnemonic: -m --mnemonic <MNEMONIC> "BIP39 mnemonic phrase of signing account")
            .conflicts_with("key"),
          arg!(amount: -a --amount <AMOUNT> "Amount to transfer in POLYX")
            .value_parser(value_parser!(f64))
            .required(true),
          arg!(destination: -d --destination <DESTINATION> "Public SS58 address of receiving account")
            .required(true),
          arg!(mainnet: --mainnet "If set, performs transaction on mainnet").required(false),
        ])
    )

    // Subcommand: sign
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

    // Subcommand: verify
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

    // Subcommand: address
    .subcommand(
      Command::new("address")
        .about("Get a user's public address given their hexadecimal private key")
        .args(&[
          arg!(key: "32-byte hexadecimal private key of signing account").required_unless_present("mnemonic"),
          arg!(mainnet: --mainnet "If set, returns mainnet address (starting with 2)").required(false),
          arg!(mnemonic: -m --mnemonic <MNEMONIC> "Use BIP39 mnemonic rather than hexadecimal private key").conflicts_with("key"),
        ])
    )

    // Subcommand: balance
    .subcommand(
      Command::new("balance")
        .about("Get a user's balance on mainnet or testnet (in μPOLYX)")
        .args(&[
          arg!(address: "SS58-formatted public address (starts with 5 on testnet or 2 on mainnet)").required(true),
          arg!(staked: -s --staked "If set, return the staked balance"),
          arg!(mainnet: --mainnet "If set, returns mainnet address (starting with 2)").required(false),
        ])
    )

    // Subcommand: secondary
    .subcommand(
      Command::new("secondary")
        .about("Add and remove secondary keys from primary signing account.")
        .subcommand_required(true)
        .subcommand(
          Command::new("add")
            .about("Add a secondary key with authorizations to an identity (must be signed by primary key)")
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
            .about("Remove a secondary key from an identity (must be signed by primary key)")
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

    // Subcommand: staking
    .subcommand(
      Command::new("staking")
        .about("Staking utilities")
        .subcommand_required(true)
        .subcommand(
          Command::new("validators")
            .about("Get public (SS58-formatted) addresses of current validator nodes")
            .short_flag('v')
            .arg(arg!(mainnet: --mainnet "If set, gets operator nodes on mainnet"))
        )
        .subcommand(
          Command::new("nominate")
            .about("Declare to nominate validator nodes for the origin controller")
            .short_flag('n')
            .args(&[
              arg!(key: -k --key <CONTROLLER_KEY> "The 32-byte hexadecimal signing key of controller account")
                .alias("controller")
                .short_alias('c')
                .required(true),
              arg!(validators: -v --validators <VALIDATORS> "The validator nodes to nominate (up to 24)")
                .alias("operators")
                .short_alias('o')
                .num_args(1..=24)   // You can only nominate up to 24 validator nodes
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
        .subcommand(
          Command::new("bond")
            .about("Take the origin account as a stash, locking up some of its balance for staking")
            .short_flag('b')
            .args(&[
              arg!(key: -k --key <STASH_KEY> "The 32-byte hexadecimal signing key of stash")
                .alias("stash")
                .short_alias('s')
                .required(true),
              arg!(controller: -c --controller <CONTROLLER_ADDR> "The public address of the controller account")
                .required(true),
              arg!(value: -v --value <VALUE> "The amount of the stash's balance (in POLYX) that will be locked up")
                .alias("amount")
                .short_alias('a')
                .value_parser(value_parser!(f64))
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
        .subcommand(
          Command::new("unbond")
            .about("As a controller, unbond an amount that has been bonded to you for staking.")
            .short_flag('u')
            .args(&[
              arg!(key: -k --key <CONTROLLER_KEY> "The 32-byte hexadecimal signing key of controller")
                .alias("controller")
                .short_alias('c')
                .required(true),
              arg!(value: -v --value <VALUE> "The amount of the stash's balance (in POLYX) that will be unbonded")
                .alias("amount")
                .short_alias('a')
                .value_parser(value_parser!(f64))
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
        .subcommand(
          Command::new("extra")
            .about("Take the origin account as a stash, bonding an extra amount for staking rewards")
            .short_flag('e')
            .args(&[
              arg!(key: -k --key <STASH_KEY> "The 32-byte hexadecimal signing key of stash")
                .alias("stash")
                .short_alias('s')
                .required(true),
              arg!(value: -v --value <VALUE> "The amount of the stash's balance (in POLYX) that will be locked up (with no upper limit on this amount)")
                .alias("amount")
                .short_alias('a')
                .value_parser(value_parser!(f64))
                .required(true),
              arg!(mainnet: --mainnet "If set, performs action on mainnet").required(false),
            ])
        )
        .subcommand(
          Command::new("withdraw")
            .about("Taking the origin as a controller, withdraw unbonded tokens (if the unbonding period has ended)")
            .short_flag('w')
            .args(&[
              arg!(key: -k --key <CONTROLLER_KEY> "The 32-byte hexadecimal signing key of the controller account").required(true),
              arg!(mainnet: -m --mainnet "If set, performs the action on mainnet").required(false),
            ])
        )
    )
}
