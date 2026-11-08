pub mod app;
pub mod controllers;
pub mod data;
pub mod dtos;
pub mod initializers;
pub mod mailers;
pub mod models;
pub mod tasks;
pub mod views;
pub mod workers;

/// 单元测试启动前加载项目根目录的 `.env`。
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn init_unit_test_env() {
    let _ = dotenvy::dotenv();
}
