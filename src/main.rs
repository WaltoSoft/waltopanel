use curtain_bar::app;

fn main() {
  if let Err(e) = app::run() {
    eprintln!("Application error: {}", e);
    std::process::exit(1);
  }
}
