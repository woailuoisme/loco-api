//! # OpenTelemetry 日志上报 (Logger) 初始化模块
//!
//! 负责构建基于 OTLP HTTP 的 `SdkLoggerProvider`，并提供 Tracing 桥接 Layer。

use std::time::Duration;

use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};

use crate::common::telemetry::{config::OtelConfig, resource};

/// 初始化日志管线并返回 SdkLoggerProvider 与 Tracing 桥接层
pub fn init_logger_provider(
    config: &OtelConfig,
) -> Result<
    (
        SdkLoggerProvider,
        OpenTelemetryTracingBridge<SdkLoggerProvider, opentelemetry_sdk::logs::SdkLogger>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(&config.endpoint)
        .with_timeout(Duration::from_secs(config.export_timeout_secs))
        .build()?;

    let resource = resource::build_resource(config);
    let batch_processor = BatchLogProcessor::builder(exporter).build();

    let logger_provider = SdkLoggerProvider::builder()
        .with_log_processor(batch_processor)
        .with_resource(resource)
        .build();

    let bridge = OpenTelemetryTracingBridge::new(&logger_provider);

    Ok((logger_provider, bridge))
}
