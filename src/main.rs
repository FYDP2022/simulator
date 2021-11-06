mod application;
mod featuredb;
mod gfx;
mod net;
mod raycast;
#[allow(dead_code)]
mod trackball;
mod ui;

use application::Application;

#[async_std::main]
async fn main() {
  let app = Application::new().await;
  app.run().await;
}
