//! # Centrifugo 实时推送控制器 (Realtime Controller)
//!
//! 提供客户端连接凭据与频道订阅凭据的获取接口。

use std::sync::Arc;

use axum::http::StatusCode;
use loco_rs::prelude::*;

use crate::{
    common::{
        consts::{api::API_PREFIX, openapi::TAG_REALTIME},
        realtime::{RealtimeChannel, RealtimeService},
    },
    controllers::auth::AuthSession,
    dtos::realtime::{ChannelSubscriptionRequest, ChannelSubscriptionResource, RealtimeTokenResource},
    errors::{AppError, AppResult},
};

/// 获取 Centrifugo WebSocket 客户端连接凭证
#[utoipa::path(
    get,
    path = "/v1/realtime/token",
    summary = "获取实时连接凭据",
    description = "为当前已认证用户生成 Centrifugo 客户端 WebSocket 连接 JWT 凭证",
    responses(
        (status = 200, description = "获取成功", body = RealtimeTokenResource),
        (status = 401, description = "未授权 / 登录已过期"),
        (status = 503, description = "实时服务未就绪")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_REALTIME
)]
#[debug_handler]
pub async fn get_connection_token(
    auth: AuthSession,
    State(ctx): State<AppContext>,
) -> AppResult<Json<RealtimeTokenResource>> {
    let Some(realtime) = ctx.shared_store.get::<Arc<RealtimeService>>() else {
        return Err(AppError::Custom(
            StatusCode::SERVICE_UNAVAILABLE,
            "Realtime service is not configured or disabled".to_string(),
        ));
    };

    let token = realtime.generate_connection_token(&auth.user.id, None)?;
    let expires_in = realtime.config().token_ttl_secs;
    let ws_url = realtime.config().ws_url.clone();

    Ok(Json(RealtimeTokenResource {
        token,
        ws_url,
        expires_in,
        user_id: auth.user.id,
    }))
}

/// 获取指定频道的订阅凭证 (Subscription Token)
#[utoipa::path(
    post,
    path = "/v1/realtime/subscription-token",
    summary = "获取频道订阅凭据",
    description = "校验频道访问权限并为当前用户签署目标频道的订阅 JWT 凭证",
    request_body = ChannelSubscriptionRequest,
    responses(
        (status = 200, description = "获取成功", body = ChannelSubscriptionResource),
        (status = 400, description = "参数验证错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权订阅目标频道"),
        (status = 503, description = "实时服务未就绪")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_REALTIME
)]
#[debug_handler]
pub async fn get_subscription_token(
    auth: AuthSession,
    State(ctx): State<AppContext>,
    JsonValidate(req): JsonValidate<ChannelSubscriptionRequest>,
) -> AppResult<Json<ChannelSubscriptionResource>> {
    let Some(realtime) = ctx.shared_store.get::<Arc<RealtimeService>>() else {
        return Err(AppError::Custom(
            StatusCode::SERVICE_UNAVAILABLE,
            "Realtime service is not configured or disabled".to_string(),
        ));
    };

    // 委托给 RealtimeChannel 领域对象进行结构解析与订阅授权判定
    let channel = RealtimeChannel::parse(&req.channel).ok_or_else(|| {
        AppError::Custom(
            StatusCode::FORBIDDEN,
            "Unsupported or unauthorized channel prefix".to_string(),
        )
    })?;

    let is_authorized = channel
        .can_subscribe(&auth.user, &ctx.db)
        .await
        .map_err(AppError::from)?;

    if !is_authorized {
        return Err(AppError::Custom(
            StatusCode::FORBIDDEN,
            "Access denied to the requested channel".to_string(),
        ));
    }

    let token = realtime.generate_subscription_token(&auth.user.id, &req.channel, None)?;
    let expires_in = realtime.config().token_ttl_secs;

    Ok(Json(ChannelSubscriptionResource {
        token,
        channel: req.channel,
        expires_in,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix(&format!("{API_PREFIX}/realtime"))
        .add("/token", get(get_connection_token))
        .add("/subscription-token", post(get_subscription_token))
}
