use anyhow::Result;

mod command;
mod util;

mod address;
mod balance;
mod secondary;
mod signing;
mod staking;
mod transaction;

pub async fn run() -> Result<String> {
  let res = match command::command().get_matches().subcommand() {
    // Subcommand: send
    Some(("send", sub_m)) => {
      let amount_polyx = sub_m.get_one::<f64>("amount").expect("amount required");
      let amount = (*amount_polyx * 1e6) as u128; // convert POLYX to μPOLYX
      let destination = sub_m
        .get_one::<String>("destination")
        .expect("destination required");
      let mainnet = sub_m.get_flag("mainnet");
      match sub_m.get_one::<String>("key") {
        Some(key) => transaction::withdraw(key, destination, amount, mainnet).await?,
        None => {
          let mnemonic = sub_m
            .get_one::<String>("mnemonic")
            .expect("requires either key or mnemonic");
          transaction::withdraw_with_mnemonic(mnemonic, destination, amount, mainnet).await?
        }
      }
    }

    // Subcommand: sign
    Some(("sign", sub_m)) => {
      let key = sub_m.get_one::<String>("key").expect("key required");
      let payload = sub_m.get_one::<String>("payload").expect("key required");
      signing::sign_payload(key, payload).await?
    }

    // Subcommand: verify
    Some(("verify", sub_m)) => {
      let address = sub_m
        .get_one::<String>("address")
        .expect("address required");
      let payload = sub_m
        .get_one::<String>("payload")
        .expect("payload required");
      let signature = sub_m
        .get_one::<String>("signature")
        .expect("signature required");
      signing::verify_signature(signature, address, payload).to_string()
    }

    // Subcommand: address
    Some(("address", sub_m)) => {
      let mainnet = sub_m.get_flag("mainnet");
      match sub_m.get_one::<String>("mnemonic") {
        Some(mnemonic) => address::mnemonic_to_ss58check(mnemonic, mainnet, None)?,
        None => {
          let key = sub_m.get_one::<String>("key").expect("key required");
          address::private_key_to_ss58check(key, mainnet)?
        }
      }
    }

    // Subcommand: balance
    Some(("balance", sub_m)) => {
      let address = sub_m
        .get_one::<String>("address")
        .expect("address required");
      let mainnet = sub_m.get_flag("mainnet");
      let bal = if sub_m.get_flag("staked") {
        balance::staked(address, mainnet).await?
      } else {
        balance::free(address, mainnet).await?
      };
      format!("{bal}",)
    }

    // Subcommand: secondary (i.e. Secondary keys)
    Some(("secondary", sub_m)) => match sub_m.subcommand() {
      Some(("add", sub_m)) => {
        let mnemonic = sub_m
          .get_one::<String>("mnemonic")
          .expect("mnemonic required");
        let secondary_key = sub_m
          .get_one::<String>("secondary_key")
          .expect("secondary key required");
        let expires_after = sub_m
          .get_one::<u64>("expires_after")
          .expect("expiry required");
        let mainnet = sub_m.get_flag("mainnet");
        secondary::add(mnemonic, secondary_key, *expires_after, mainnet).await?
      }
      Some(("remove", sub_m)) => {
        let mnemonic = sub_m
          .get_one::<String>("mnemonic")
          .expect("mnemonic required");
        let who = sub_m.get_one::<String>("who").expect("who required");
        let mainnet = sub_m.get_flag("mainnet");
        secondary::remove(mnemonic, who, mainnet).await?
      }
      _ => unreachable!(),
    },

    // Subcommand: staking
    Some(("staking", sub_m)) => match sub_m.subcommand() {
      Some(("validators", sub_m)) => {
        let mainnet = sub_m.get_flag("mainnet");
        let operators = staking::validators(mainnet).await?;
        operators
          .iter()
          .fold(String::new(), |acc, operator| acc + operator + "\n")
          .strip_suffix('\n')
          .expect("there should be a new line at the end of iter")
          .to_string()
      }
      Some(("nominate", sub_m)) => {
        let controller_key = sub_m
          .get_one::<String>("key")
          .expect("controller key required");
        let validators: Vec<&str> = sub_m
          .get_many::<String>("validators")
          .expect("validators required")
          .map(|s| s.as_str())
          .collect();
        let mainnet = sub_m.get_flag("mainnet");
        staking::nominate(controller_key, validators, mainnet).await?
      }
      Some(("bond", sub_m)) => {
        let stash_key = sub_m.get_one::<String>("key").expect("stash key required");
        let controller = sub_m
          .get_one::<String>("controller")
          .expect("controller address required");
        let value_polyx = sub_m.get_one::<f64>("value").expect("value required");
        let value = (*value_polyx * 1e6) as u128; // convert POLYX to μPOLYX
        let mainnet = sub_m.get_flag("mainnet");
        staking::bond(stash_key, controller, value, mainnet).await?
      }
      Some(("unbond", sub_m)) => {
        let controller_key = sub_m
          .get_one::<String>("key")
          .expect("controller key required");
        let value_polyx = sub_m.get_one::<f64>("value").expect("value required");
        let value = (*value_polyx * 1e6) as u128; // convert POLYX to μPOLYX
        let mainnet = sub_m.get_flag("mainnet");
        staking::unbond(controller_key, value, mainnet).await?
      }
      Some(("extra", sub_m)) => {
        let stash_key = sub_m.get_one::<String>("key").expect("stash key required");
        let value_polyx = sub_m.get_one::<f64>("value").expect("value required");
        let value = (*value_polyx * 1e6) as u128; // convert POLYX to μPOLYX
        let mainnet = sub_m.get_flag("mainnet");
        staking::bond_extra(stash_key, value, mainnet).await?
      }
      Some(("withdraw", sub_m)) => {
        let controller_key = sub_m
          .get_one::<String>("key")
          .expect("controller key required");
        let mainnet = sub_m.get_flag("mainnet");
        staking::withdraw_unbonded(controller_key, mainnet).await?
      }
      _ => unreachable!(), // subcommand required
    },
    _ => unreachable!(), // subcommand required
  };

  Ok(res)
}
