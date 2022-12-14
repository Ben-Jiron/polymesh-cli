use std::io::{self, Write};

#[tokio::main]
async fn main() {
  let mut stdout = io::stdout().lock();
  let mut stderr = io::stderr().lock();
  match polymesh_cli::run().await {
    Ok(res) => write!(stdout, "{}", res).expect("failed to write to stdout"),
    Err(e) => {
      write!(stderr, "{}", e).expect("failed to write to stderr");
      std::process::exit(1);
    }
  }
  stdout.flush().expect("failed to flush stdout");
  stderr.flush().expect("failed to flush stdout");
}
