use anyhow::Result;

mod command;
mod util;

mod address;
mod secondary;
mod signing;
mod transaction;

pub async fn run() -> Result<String> {
  let res = match command::command().get_matches().subcommand() {
    Some(("send", sub_m)) => {
      let amount = sub_m.get_one::<f64>("amount").expect("amount required");
      let amount = (*amount * 1e6) as u128;
      let key = sub_m.get_one::<String>("key").expect("key required");
      let destination = sub_m
        .get_one::<String>("destination")
        .expect("destination required");
      let mainnet = sub_m.get_flag("mainnet");
      transaction::withdraw(key, destination, amount, mainnet).await?
    }
    Some(("sign", sub_m)) => {
      let key = sub_m.get_one::<String>("key").expect("key required");
      let payload = sub_m.get_one::<String>("payload").expect("key required");
      signing::sign_payload(key, payload).await?
    }
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
    Some(("address", sub_m)) => {
      let key = sub_m.get_one::<String>("key").expect("key required");
      let mainnet = sub_m.get_flag("mainnet");
      address::private_key_to_ss58check(key, mainnet)?
    }
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
    _ => unreachable!(),
  };

  Ok(res)
}
