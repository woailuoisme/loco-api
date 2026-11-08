use loco_api::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    loco_api::env_loader::load_env();

    cli::main::<App, Migrator>().await
}
