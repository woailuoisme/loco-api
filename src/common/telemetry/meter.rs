//! # OpenTelemetry 指标监控 (Metrics) 初始化模块
//!
//! 负责构建基于 OTLP HTTP 的 `SdkMeterProvider`，定义符合 OpenTelemetry 语义约定的 HTTP 服务核心指标。

use std::{sync::OnceLock, time::Duration};

use opentelemetry::metrics::{Counter, Histogram, MeterProvider, UpDownCounter};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::metrics::SdkMeterProvider;

use crate::common::telemetry::{config::OtelConfig, resource};

/// 全局 HTTP 核心指标容器
pub static HTTP_METRICS: OnceLock<HttpMetrics> = OnceLock::new();

/// HTTP 监控指标项
#[derive(Clone, Debug)]
pub struct HttpMetrics {
    /// HTTP 请求耗时直方图 (单位: 秒)
    pub duration: Histogram<f64>,
    /// 当前正在处理中的活跃 HTTP 请求数
    pub active_requests: UpDownCounter<i64>,
    /// HTTP 请求累计总数
    pub total_requests: Counter<u64>,
}

impl HttpMetrics {
    /// 基于给定 Meter 创建标准 HTTP 监控指标
    pub fn new(meter: &opentelemetry::metrics::Meter) -> Self {
        let duration = meter
            .f64_histogram("http.server.request.duration")
            .with_description("HTTP server request duration in seconds")
            .with_unit("s")
            .build();

        let active_requests = meter
            .i64_up_down_counter("http.server.active_requests")
            .with_description("Number of active HTTP server requests")
            .with_unit("{request}")
            .build();

        let total_requests = meter
            .u64_counter("http.server.request.total")
            .with_description("Total number of HTTP server requests")
            .with_unit("{request}")
            .build();

        Self {
            duration,
            active_requests,
            total_requests,
        }
    }
}

/// 初始化指标管线并设置全局 HTTP 指标
pub fn init_meter_provider(config: &OtelConfig) -> Result<SdkMeterProvider, Box<dyn std::error::Error + Send + Sync>> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(&config.endpoint)
        .with_timeout(Duration::from_secs(config.export_timeout_secs))
        .build()?;

    let resource = resource::build_resource(config);
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build();

    let meter = meter_provider.meter("loco-api");
    let _ = HTTP_METRICS.set(HttpMetrics::new(&meter));

    Ok(meter_provider)
}
