//! # OpenTelemetry 分布式追踪 (Tracer) 初始化模块
//!
//! 负责构建基于 OTLP HTTP 的 `SdkTracerProvider`，并导出当前服务的 Tracer 实例。

use std::time::Duration;

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::{BatchSpanProcessor, Sampler, SdkTracerProvider, Tracer};

use crate::common::telemetry::{config::OtelConfig, resource};

/// 初始化分布式追踪管线并返回 SdkTracerProvider 与 Tracer
pub fn init_tracer_provider(
    config: &OtelConfig,
) -> Result<(SdkTracerProvider, Tracer), Box<dyn std::error::Error + Send + Sync>> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(&config.endpoint)
        .with_timeout(Duration::from_secs(config.export_timeout_secs))
        .build()?;

    let sampler = if config.sample_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sample_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(config.sample_ratio)))
    };

    let resource = resource::build_resource(config);
    let batch_processor = BatchSpanProcessor::builder(exporter).build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_span_processor(batch_processor)
        .with_sampler(sampler)
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());

    Ok((tracer_provider, tracer))
}
