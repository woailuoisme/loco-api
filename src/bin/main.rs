use loco_api::app::App;
use loco_rs::cli;
use migration::Migrator;

/// 启动 CLI：先加载项目根目录的 `.env`，再交给 Loco 处理。
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    let _ = dotenvy::dotenv();

    cli::main::<App, Migrator>().await
}
