/// 集成测试启动前加载项目根目录的 `.env`。
#[ctor::ctor(unsafe)]
fn init_test_env() {
    let _ = dotenvy::dotenv();
}

mod models;
mod requests;
mod tasks;
mod workers;
