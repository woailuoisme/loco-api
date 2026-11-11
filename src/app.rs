//! # 应用核心入口与 Hooks 接口
//!
//! 该模块定义了 Loco 全栈应用的入口结构体 `App`，实现 `Hooks` Trait。
//! 所有具体的路由、中间件、Task、Worker 注册逻辑均已委托给 [`crate::setup`] 子模块办理。

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    Result,
    app::{AppContext, Hooks, Initializer, serve_router},
    bgworker::Queue,
    boot::{BootResult, ServeParams, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    task::Tasks,
};
use migration::Migrator;
use std::path::Path;

use crate::{
    models::_entities::{account, session, user},
    setup,
};

/// Loco 应用主结构体
pub struct App;

#[async_trait]
impl Hooks for App {
    /// 返回当前应用版本号与 Git Hash 标识
    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA").or(option_env!("GITHUB_SHA")).unwrap_or("dev")
        )
    }

    /// 返回应用 Crate 名称
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    /// 初始化系统日志与链路追踪系统
    ///
    /// 若启用 OpenTelemetry，则装配 Traces/Metrics/Logs 导出器并返回 `Ok(true)`；
    /// 否则返回 `Ok(false)` 由 Loco 默认控制台格式化日志接管。
    fn init_logger(ctx: &AppContext) -> Result<bool> {
        crate::common::telemetry::init(ctx)
    }

    /// 引导启动底层网络服务与数据库迁移驱动
    async fn boot(mode: StartMode, environment: &Environment, config: Config) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    /// 启动 Axum Web 服务并输出精简结构化启动日志 (替代冗长 ASCII 字符画 Banner)
    async fn serve(app: AxumRouter, ctx: &AppContext, serve_params: &ServeParams) -> Result<()> {
        tracing::info!(
            environment = %ctx.environment,
            logger = %ctx.config.logger.level,
            compilation = if cfg!(debug_assertions) { "debug" } else { "release" },
            modes = "server",
            listening = %format!("http://{}:{}", serve_params.binding, serve_params.port),
            "Server ready"
        );

        let ctx_clone = ctx.clone();
        serve_router(
            app,
            serve_params,
            Box::pin(async move { Self::on_shutdown(&ctx_clone).await }),
        )
        .await
    }

    /// 在 AppContext 构建完成后，装配全局基础设施与运行时设置 (委托给 `setup::storage::init` 等)
    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        crate::controllers::auth::init_cookie_security(ctx.environment == Environment::Production);

        if let Some(machine_id) = ctx
            .config
            .settings
            .as_ref()
            .and_then(|s| s.get("system"))
            .and_then(|s| s.get("machine_id"))
            .and_then(|v| v.as_u64())
        {
            crate::common::utils::order_no::set_default_machine_id(machine_id as u32);
        }

        // 初始化并装配 Centrifugo 实时通讯服务 (DI -> SharedStore)
        if let Some(rt_cfg) = crate::common::realtime::CentrifugoConfig::from_ctx(&ctx) {
            match crate::common::realtime::RealtimeService::new(rt_cfg) {
                Ok(service) => {
                    ctx.shared_store.insert(std::sync::Arc::new(service));
                    tracing::info!("Centrifugo RealtimeService initialized and registered to shared_store");
                }
                Err(err) => {
                    tracing::warn!("Failed to initialize Centrifugo RealtimeService: {err}");
                }
            }
        }

        setup::storage::init(ctx).await
    }

    /// 绑定全局 HTTP 中间件与请求日志钩子 (委托给 `setup::middleware::register`)
    async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        setup::middleware::register(router, ctx).await
    }

    /// 加载系统 Initializer 插件 (例如 OpenAPI 接口文档、配置校验、日志系统)
    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![
            Box::new(crate::initializers::config_validator::ConfigValidatorInitializer),
            Box::new(crate::initializers::openapi::OpenApiInitializer),
        ])
    }

    /// 绑定并返回全局控制器路由 (委托给 `setup::routes`)
    fn routes(ctx: &AppContext) -> AppRoutes {
        setup::routes::routes(ctx)
    }

    /// 注册后台异步任务 Worker (委托给 `setup::workers`)
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        setup::workers::connect_workers(ctx, queue).await
    }

    /// 注册命令行 Task 列表 (委托给 `setup::tasks`)
    fn register_tasks(tasks: &mut Tasks) {
        setup::tasks::register_tasks(tasks);
    }

    /// 清空数据库数据 (主要用于测试数据重置)
    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, session::Entity).await?;
        truncate_table(&ctx.db, account::Entity).await?;
        truncate_table(&ctx.db, user::Entity).await?;
        Ok(())
    }

    /// 执行基础 Seed 种子数据注入
    async fn seed(_ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(())
    }

    /// 应用关闭停机钩子：冲刷 OpenTelemetry 遥测缓冲
    async fn on_shutdown(_ctx: &AppContext) {
        crate::common::telemetry::shutdown();
    }
}
