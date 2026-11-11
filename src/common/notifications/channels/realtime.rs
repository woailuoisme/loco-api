//! # Centrifugo 实时推送渠道实现 (Realtime Channel)

use std::sync::Arc;

use loco_rs::prelude::*;

use crate::common::realtime::{RealtimeMessage, RealtimeService};

/// Centrifugo 实时推送渠道 (RealtimeChannel)
pub struct RealtimeChannel;

impl RealtimeChannel {
    /// 通过 Centrifugo 向目标频道推送实时事件
    pub async fn send(ctx: &AppContext, default_channel: &str, message: &RealtimeMessage) -> Result<()> {
        let Some(realtime) = ctx.shared_store.get::<Arc<RealtimeService>>() else {
            tracing::warn!("Centrifugo RealtimeService 未在 shared_store 中注册，跳过实时消息推送");
            return Ok(());
        };

        // 若信封中的 channel 为空，则回退使用 notifiable 路由的目标频道
        let mut msg = message.clone();
        if msg.channel.is_empty() && !default_channel.is_empty() {
            msg.channel = default_channel.to_string();
        }

        if msg.channel.is_empty() {
            tracing::warn!("跳过实时消息推送: 目标 Channel 为空");
            return Ok(());
        }

        realtime.publish_message(&msg).await.map_err(|e| {
            tracing::error!("Centrifugo 实时推送失败 -> channel({}): {:?}", msg.channel, e);
            Error::string(&format!("Realtime publish error: {e}"))
        })?;

        tracing::info!(
            "Centrifugo 实时消息成功推送 -> channel({}), event({})",
            msg.channel,
            msg.event
        );
        Ok(())
    }
}
