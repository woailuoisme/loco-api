//! # 统一实时消息事件信封 (Realtime Message Envelope)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 统一实时事件信封，确保所有跨频道消息格式的一致性与可追踪性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeMessage {
    /// 消息全局唯一追踪 ID (UUID v4)
    pub id: String,
    /// 事件类型名称 (例如: "notification.created", "system.alert")
    pub event: String,
    /// 目标频道名称 (例如: "personal:123", "org:45", "broadcast:system")
    pub channel: String,
    /// 事件业务负载数据
    pub payload: serde_json::Value,
    /// 事件产生时间戳 (UTC)
    pub timestamp: DateTime<Utc>,
}

impl RealtimeMessage {
    /// 创建新的实时事件消息信封
    pub fn new(channel: impl Into<String>, event: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event: event.into(),
            channel: channel.into(),
            payload,
            timestamp: Utc::now(),
        }
    }

    /// 快捷构建个人私有频道名称: `user:{user_id}` (对齐 Centrifugo 原生 `user` 命名空间)
    pub fn personal_channel(user_id: impl std::fmt::Display) -> String {
        format!("user:{user_id}")
    }

    /// 快捷构建组织团队频道名称: `private:org_{org_id}` (对齐 Centrifugo 原生 `private` 命名空间)
    pub fn org_channel(org_id: impl std::fmt::Display) -> String {
        format!("private:org_{org_id}")
    }

    /// 全局系统广播频道: `public:system` (对齐 Centrifugo 原生 `public` 命名空间)
    pub const BROADCAST_SYSTEM: &'static str = "public:system";
}
