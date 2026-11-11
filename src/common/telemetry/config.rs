//! # OpenTelemetry 配置结构体
//!
//! 定义 OpenTelemetry 遥测三位一体（Logs, Metrics, Traces）的核心配置，
//! 支持从 Loco 的 `AppContext` 中解析 `settings.opentelemetry` 节点。

use loco_rs::app::AppContext;
use serde::{Deserialize, Serialize};

/// OpenTelemetry 核心配置
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OtelConfig {
    /// 是否开启遥测导出 (默认 false)
    #[serde(default)]
    pub enabled: bool,

    /// OTLP HTTP 导出端点 (默认 "http://127.0.0.1:4318")
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// 服务名称 (默认 "loco-api")
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// Trace 采样率 (0.0 ~ 1.0，默认 1.0 全量采样)
    #[serde(default = "default_sample_ratio")]
    pub sample_ratio: f64,

    /// 导出超时时间 (秒，默认 3)
    #[serde(default = "default_export_timeout_secs")]
    pub export_timeout_secs: u64,
}

fn default_endpoint() -> String {
    "http://127.0.0.1:4318".to_string()
}

fn default_service_name() -> String {
    "loco-api".to_string()
}

fn default_sample_ratio() -> f64 {
    1.0
}

fn default_export_timeout_secs() -> u64 {
    3
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: default_endpoint(),
            service_name: default_service_name(),
            sample_ratio: default_sample_ratio(),
            export_timeout_secs: default_export_timeout_secs(),
        }
    }
}

impl OtelConfig {
    /// 从 Loco AppContext 解析 OpenTelemetry 配置
    pub fn from_ctx(ctx: &AppContext) -> Option<Self> {
        let val = ctx.config.settings.as_ref()?.get("opentelemetry")?.clone();

        serde_json::from_value(val).ok()
    }
}
