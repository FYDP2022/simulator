mod application;
mod cli;
mod featuredb;
mod gfx;
mod net;
mod raycast;
#[allow(dead_code)]
mod trackball;
mod ui;

use application::Application;
use cli::Cli;

#[async_std::main]
async fn main() {
  let cli = Cli::new();
  if !cli.run().unwrap() {
    let app = Application::new().await;
    app.run().await;
  }
}
