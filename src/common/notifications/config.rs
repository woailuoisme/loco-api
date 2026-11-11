//! # 通知渠道配置项解析与提取 (Notification Config)

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// Telegram Channel 配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelegramConfig {
    /// Telegram Bot API Token
    pub bot_token: String,
    /// 默认 Chat ID (可选，未指定 routing 时回退)
    pub chat_id: Option<String>,
    /// Telegram Bot API 请求基准 URL (默认: `https://api.telegram.org`)
    pub api_url: String,
    /// HTTP 请求超时时间 (秒，默认: 10)
    pub timeout_sec: u64,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: None,
            api_url: "https://api.telegram.org".to_string(),
            timeout_sec: 10,
        }
    }
}

impl TelegramConfig {
    /// 从 Loco `AppContext` 的全局 Config 中的 `settings.telegram` 提取 Telegram 配置
    pub fn from_ctx(ctx: &AppContext) -> Option<Self> {
        let cfg = ctx
            .config
            .settings
            .as_ref()
            .and_then(|s| s.get("telegram"))
            .and_then(|val| serde_json::from_value::<Self>(val.clone()).ok())?;

        if !cfg.bot_token.is_empty() { Some(cfg) } else { None }
    }
}
