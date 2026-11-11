//! # Scout / Meilisearch 配置提取器 (Config)

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// Meilisearch 搜索引擎配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MeilisearchConfig {
    /// Meilisearch 服务的主机 HTTP 地址 (例如: `http://localhost:7700`)
    pub host: String,
    /// Meilisearch 主密钥 / API Key (Master Key)
    #[serde(alias = "key")]
    pub api_key: Option<String>,
    /// 索引统一前缀 (可选，如 `prod_`, `dev_`)
    #[serde(alias = "prefix")]
    pub index_prefix: Option<String>,
    /// HTTP 请求超时秒数 (默认: 10)
    pub timeout_sec: u64,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            host: "http://localhost:7700".to_string(),
            api_key: None,
            index_prefix: None,
            timeout_sec: 10,
        }
    }
}

impl MeilisearchConfig {
    /// 从 Loco `AppContext` 的全局 Config 中的 `settings.scout.meilisearch` 提取 Meilisearch 配置
    pub fn from_ctx(ctx: &AppContext) -> Option<Self> {
        let cfg = ctx
            .config
            .settings
            .as_ref()
            .and_then(|s| s.get("scout"))
            .and_then(|s| s.get("meilisearch"))
            .and_then(|v| serde_json::from_value::<Self>(v.clone()).ok())?;

        if !cfg.host.is_empty() { Some(cfg) } else { None }
    }

    /// 获取附带前缀后的最终 Meilisearch 索引名
    pub fn prefixed_index(&self, index_name: &str) -> String {
        self.index_prefix
            .as_deref()
            .map_or_else(|| index_name.to_string(), |prefix| format!("{prefix}{index_name}"))
    }
}
