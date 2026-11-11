rust_i18n::i18n!("locales");

pub mod app;
pub mod common;
pub mod controllers;
pub mod data;
pub mod dtos;
pub mod errors;
pub mod exports;
pub mod imports;
pub mod initializers;
pub mod mailers;
pub mod models;
pub mod services;
pub mod setup;
pub mod tasks;
pub mod workers;

/// 单元测试启动前加载项目根目录的 `.env`。
// SAFETY: `ctor` 在测试执行环境启动前作为全局构造器执行，
// 仅调用 `dotenvy::dotenv()` 幂等载入环境变量，不涉及未初始化的可变全局状态或数据竞争。
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn init_unit_test_env() {
    let _ = dotenvy::dotenv();
}
