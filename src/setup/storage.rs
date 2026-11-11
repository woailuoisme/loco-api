//! # 对象存储 (Storage) 基础设施装配模块
//!
//! Loco 在 `boot` 阶段已基于 `config.storage` 原生装配好全局对象存储驱动 (`ctx.storage`)。
//! 本模块在 `after_context` 钩子中仅对 `Environment::Test` 环境注入内存存储驱动 (`mem::new()`)，
//! 确保自动化单元与集成测试零磁盘与网络依赖。

use loco_rs::{
    Result,
    app::AppContext,
    environment::Environment,
    storage::{Storage, drivers::mem},
};

/// 检查并在测试环境下初始化内存对象存储驱动
pub async fn init(mut ctx: AppContext) -> Result<AppContext> {
    if ctx.environment == Environment::Test {
        tracing::info!(driver = "memory", "初始化测试环境内存对象存储驱动");
        ctx.storage = Storage::single(mem::new()).into();
    }
    Ok(ctx)
}
