//! # OpenTelemetry 遥测三位一体核心集成模块
//!
//! 提供基于 OTLP HTTP (端口 4318) 的分布式链路追踪 (Traces)、指标监控 (Metrics) 与日志桥接 (Logs) 统一管理。
//! 遵循 Loco 框架的最佳实践，通过 Hooks 拦截日志管道与优雅停机，同时支持灵活降级回退。

pub mod config;
pub mod logger;
pub mod meter;
pub mod propagator;
pub mod resource;
pub mod tracer;

use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

use loco_rs::{Result, app::AppContext};
use opentelemetry::trace::TraceContextExt;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider,
};
pub use propagator::{HeaderExtractor, HeaderInjector};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use config::OtelConfig;
pub use meter::{HTTP_METRICS, HttpMetrics};

static TRACER_PROVIDER: OnceLock<SdkTracerProvider> = OnceLock::new();
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();
static LOGGER_PROVIDER: OnceLock<SdkLoggerProvider> = OnceLock::new();
static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// 检查当前应用进程是否已激活 OpenTelemetry 遥测导出
pub fn is_enabled() -> bool {
    IS_INITIALIZED.load(Ordering::Acquire)
}

/// 获取当前处于活动状态的 OpenTelemetry Trace ID（32 位十六进制字符串）
pub fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    let cx = span.context();
    let span_ref = cx.span();
    let span_ctx = span_ref.span_context();
    if span_ctx.is_valid() {
        Some(format!("{:032x}", span_ctx.trace_id()))
    } else {
        None
    }
}

/// 初始化 OpenTelemetry 遥测管线 (Traces, Metrics, Logs)
///
/// 若配置未开启 (`enabled == false`) 或未配置，返回 `Ok(false)` 允许 Loco 使用默认控制台日志。
/// 若成功初始化三位一体遥测，返回 `Ok(true)` 接管系统的日志与追踪。
pub fn init(ctx: &AppContext) -> Result<bool> {
    let Some(config) = OtelConfig::from_ctx(ctx) else {
        return Ok(false);
    };

    if !config.enabled {
        return Ok(false);
    }

    // 1. 设置全局 W3C TraceContext 传播器
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    // 2. 初始化 TracerProvider 与 Tracer
    let (tracer_provider, tracer) = match tracer::init_tracer_provider(&config) {
        Ok(res) => res,
        Err(err) => {
            tracing::warn!("Failed to initialize OpenTelemetry TracerProvider: {err}");
            return Ok(false);
        }
    };

    // 3. 初始化 MeterProvider
    let meter_provider = match meter::init_meter_provider(&config) {
        Ok(res) => res,
        Err(err) => {
            tracing::warn!("Failed to initialize OpenTelemetry MeterProvider: {err}");
            return Ok(false);
        }
    };

    // 4. 初始化 LoggerProvider 与 Tracing Bridge
    let (logger_provider, log_bridge) = match logger::init_logger_provider(&config) {
        Ok(res) => res,
        Err(err) => {
            tracing::warn!("Failed to initialize OpenTelemetry LoggerProvider: {err}");
            return Ok(false);
        }
    };

    // 5. 设置全局 Provider
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // 6. 保存全局引用以便在 on_shutdown 时刷新
    let _ = TRACER_PROVIDER.set(tracer_provider);
    let _ = METER_PROVIDER.set(meter_provider);
    let _ = LOGGER_PROVIDER.set(logger_provider);

    // 7. 组装 tracing_subscriber Registry (严格遵循 Loco 声明式过滤器约定)
    let filter = loco_rs::logger::init_env_filter::<crate::app::App>(
        ctx.config.logger.override_filter.as_ref(),
        &ctx.config.logger.level,
    );

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true).compact();

    let tracer_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(tracer_layer)
        .with(log_bridge);

    if let Err(err) = registry.try_init() {
        tracing::warn!("Tracing subscriber already initialized or failed: {err}");
        return Ok(false);
    }

    IS_INITIALIZED.store(true, Ordering::Release);
    tracing::info!(
        service.name = %config.service_name,
        endpoint = %config.endpoint,
        "OpenTelemetry telemetry trinity (Traces, Metrics, Logs) successfully initialized"
    );

    Ok(true)
}

/// 优雅停机并冲刷所有遥测缓冲数据
pub fn shutdown() {
    if !is_enabled() {
        return;
    }

    // 冲刷 Tracer 缓冲
    if let Some(tp) = TRACER_PROVIDER.get() {
        let _ = tp.shutdown();
    }

    // 冲刷 Meter 缓冲
    if let Some(mp) = METER_PROVIDER.get() {
        let _ = mp.shutdown();
    }

    // 冲刷 Logger 缓冲
    if let Some(lp) = LOGGER_PROVIDER.get() {
        let _ = lp.shutdown();
    }

    IS_INITIALIZED.store(false, Ordering::Release);
}
