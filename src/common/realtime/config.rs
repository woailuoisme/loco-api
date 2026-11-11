//! # Centrifugo 实时通讯配置解析 (Centrifugo Config)

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// Centrifugo 实时通讯配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CentrifugoConfig {
    /// Centrifugo HTTP Server API 端点 (例如: `http://localhost:8000/api`)
    pub api_endpoint: String,
    /// Centrifugo HTTP Server API 认证密钥
    pub api_key: String,
    /// Centrifugo 客户端 JWT 签名 HMAC Secret
    pub token_hmac_secret: String,
    /// 客户端 JWT 校验受众 Audience (默认: `centrifugo`)
    pub token_audience: String,
    /// 前端客户端 WebSocket 连接 URL (例如: `ws://localhost:8000/connection/websocket`)
    pub ws_url: String,
    /// Client JWT 默认过期时间 (秒，默认: 86400 即 24 小时)
    pub token_ttl_secs: u64,
    /// HTTP 请求超时时间 (秒，默认: 5)
    pub timeout_sec: u64,
}

impl Default for CentrifugoConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "http://localhost:8000/api".to_string(),
            api_key: String::new(),
            token_hmac_secret: String::new(),
            token_audience: "centrifugo".to_string(),
            ws_url: "ws://localhost:8000/connection/websocket".to_string(),
            token_ttl_secs: 86400,
            timeout_sec: 5,
        }
    }
}

impl CentrifugoConfig {
    /// 从 Loco `AppContext` 的全局 Config 中的 `settings.centrifugo` 提取配置
    pub fn from_ctx(ctx: &AppContext) -> Option<Self> {
        ctx.config
            .settings
            .as_ref()
            .and_then(|s| s.get("centrifugo"))
            .and_then(|val| serde_json::from_value::<Self>(val.clone()).ok())
    }

    /// 检查是否已有效配置实时通讯服务端端点与密钥
    pub fn is_configured(&self) -> bool {
        !self.api_endpoint.is_empty() && !self.token_hmac_secret.is_empty()
    }
}
