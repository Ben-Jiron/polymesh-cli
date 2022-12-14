#[tokio::main]
async fn main() {
  match polymesh_cli::run().await {
    Ok(res) => println!("{}", res),
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  }
}
