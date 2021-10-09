mod application;
mod featuredb;
mod net;

use application::Application;

#[async_std::main]
async fn main() {
  let app = Application::new().await;
  app.run().await;
}
