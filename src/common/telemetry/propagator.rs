//! # W3C TraceContext 传播器
//!
//! 提供基于 Axum HTTP `HeaderMap` 的 W3C TraceContext 注入与提取适配器。

use axum::http::HeaderMap;
use opentelemetry::propagation::{Extractor, Injector};

/// 从 HTTP 请求头中提取链路传播信息的提取器
pub struct HeaderExtractor<'a>(pub &'a HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// 向 HTTP 响应头或外发请求头中注入链路传播信息的注入器
pub struct HeaderInjector<'a>(pub &'a mut HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(val)) = (
            axum::http::header::HeaderName::from_bytes(key.as_bytes()),
            axum::http::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, val);
        }
    }
}
