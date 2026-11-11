//! # Centrifugo 实时通讯数据传输对象 (Realtime DTOs)

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 客户端 WebSocket 连接凭证响应资源
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RealtimeTokenResource {
    /// 签名有效的 Client JWT 凭据
    pub token: String,
    /// 前端直连的 WebSocket 服务地址
    pub ws_url: String,
    /// Token 有效期 (秒)
    pub expires_in: u64,
    /// 当前认证用户全局唯一标识
    pub user_id: String,
}

/// 频道订阅凭据申请请求载荷
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChannelSubscriptionRequest {
    /// 目标频道名称 (例如 `personal:123`, `org:45`, `broadcast:system`)
    #[validate(length(min = 1, message = "Channel name cannot be empty"))]
    pub channel: String,
}

/// 频道订阅凭据响应资源
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChannelSubscriptionResource {
    /// 签名有效的 Subscription JWT 凭据
    pub token: String,
    /// 允许订阅的目标频道
    pub channel: String,
    /// 订阅 Token 有效期 (秒)
    pub expires_in: u64,
}
