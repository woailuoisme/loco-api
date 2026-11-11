//! # OpenTelemetry 共享资源 (Resource) 构造器
//!
//! 提供统一的 OpenTelemetry Resource 构造工厂，包含标准服务名称、版本号与语言属性。

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

use crate::common::telemetry::config::OtelConfig;

/// 构建包含标准语义约定的 OpenTelemetry 共享资源实体
pub fn build_resource(config: &OtelConfig) -> Resource {
    Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("telemetry.sdk.language", "rust"),
        ])
        .build()
}
