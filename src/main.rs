#[tokio::main]
async fn main() {
  match polymesh_cli::run().await {
    Ok(tx_hash) => println!("{}", tx_hash),
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  }
}
