//! # Centrifugo 实时推送控制器集成测试 (Realtime Requests Tests)

use axum::http::StatusCode;
use loco_api::{
    app::App,
    dtos::realtime::{ChannelSubscriptionRequest, ChannelSubscriptionResource, RealtimeTokenResource},
};
use loco_rs::testing::prelude::*;

#[tokio::test]
#[serial_test::serial]
async fn connection_token_requires_authentication() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/v1/realtime/token").await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn can_get_connection_token_when_authenticated() {
    request::<App, _, _>(|request, ctx| async move {
        let auth = super::prepare_data::init_user_login_unique(&request, &ctx).await;

        let res = request
            .get("/v1/realtime/token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .await;

        res.assert_status(StatusCode::OK);

        let data: RealtimeTokenResource = serde_json::from_str(&res.text()).unwrap();
        assert!(!data.token.is_empty(), "Token must not be empty");
        assert_eq!(data.user_id, auth.user.id);
        assert_eq!(data.expires_in, 3600);
        assert_eq!(data.ws_url, "ws://localhost:8000/connection/websocket");
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn subscription_token_requires_authentication() {
    request::<App, _, _>(|request, _ctx| async move {
        let req_payload = ChannelSubscriptionRequest {
            channel: "broadcast:system".to_string(),
        };
        let res = request.post("/v1/realtime/subscription-token").json(&req_payload).await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn can_get_subscription_token_for_personal_and_broadcast_channels() {
    request::<App, _, _>(|request, ctx| async move {
        let auth = super::prepare_data::init_user_login_unique(&request, &ctx).await;
        let user_channel = format!("user:{}", auth.user.id);
        let personal_channel = format!("personal:{}", auth.user.id);

        // 1. 订阅原生 user 频道 (自身) -> 成功 200
        let req_user = ChannelSubscriptionRequest {
            channel: user_channel.clone(),
        };
        let res = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req_user)
            .await;

        res.assert_status(StatusCode::OK);
        let data: ChannelSubscriptionResource = serde_json::from_str(&res.text()).unwrap();
        assert_eq!(data.channel, user_channel);
        assert!(!data.token.is_empty());
        assert_eq!(data.expires_in, 3600);

        // 2. 订阅兼容 personal 别名频道 (自身) -> 成功 200
        let req_personal = ChannelSubscriptionRequest {
            channel: personal_channel.clone(),
        };
        let res_personal = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req_personal)
            .await;
        res_personal.assert_status(StatusCode::OK);

        // 3. 订阅原生 public:system 广播频道 -> 成功 200
        let req_public = ChannelSubscriptionRequest {
            channel: "public:system".to_string(),
        };
        let res = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req_public)
            .await;

        res.assert_status(StatusCode::OK);
        let data: ChannelSubscriptionResource = serde_json::from_str(&res.text()).unwrap();
        assert_eq!(data.channel, "public:system");
        assert!(!data.token.is_empty());

        // 4. 订阅兼容 broadcast:system 别名广播频道 -> 成功 200
        let req_broadcast = ChannelSubscriptionRequest {
            channel: "broadcast:system".to_string(),
        };
        let res_broadcast = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req_broadcast)
            .await;
        res_broadcast.assert_status(StatusCode::OK);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn cannot_subscribe_to_other_users_personal_channel() {
    request::<App, _, _>(|request, ctx| async move {
        let auth = super::prepare_data::init_user_login_unique(&request, &ctx).await;
        let other_channel_user = "user:other_user_id_123456";
        let other_channel_personal = "personal:other_user_id_123456";

        let req = ChannelSubscriptionRequest {
            channel: other_channel_user.to_string(),
        };
        let res = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req)
            .await;
        res.assert_status(StatusCode::FORBIDDEN);

        let req_p = ChannelSubscriptionRequest {
            channel: other_channel_personal.to_string(),
        };
        let res_p = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req_p)
            .await;
        res_p.assert_status(StatusCode::FORBIDDEN);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn organization_channel_subscription_permission() {
    request::<App, _, _>(|request, ctx| async move {
        // 1. 创建两个独立用户
        let owner = super::prepare_data::init_user_login_unique(&request, &ctx).await;
        let non_member = super::prepare_data::init_user_login_unique(&request, &ctx).await;

        // 2. Owner 创建组织
        let slug = format!("team-{}", &uuid::Uuid::new_v4().simple().to_string()[..6]);
        let create_org_resp = request
            .post("/v1/organizations")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", owner.token))
            .json(&serde_json::json!({
                "name": "Team Realtime",
                "slug": slug
            }))
            .await;

        assert_eq!(create_org_resp.status_code(), StatusCode::OK);
        let org: loco_api::dtos::auth::OrganizationResource = serde_json::from_str(&create_org_resp.text()).unwrap();
        let org_channel = format!("private:org_{}", org.id);

        // 3. 组织成员 (Owner) 订阅该组织原生 private:org_ 频道 -> 200 OK
        let req_owner = ChannelSubscriptionRequest {
            channel: org_channel.clone(),
        };
        let res_owner = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", owner.token))
            .json(&req_owner)
            .await;

        res_owner.assert_status(StatusCode::OK);
        let data: ChannelSubscriptionResource = serde_json::from_str(&res_owner.text()).unwrap();
        assert_eq!(data.channel, org_channel);
        assert!(!data.token.is_empty());

        // 4. 非组织成员订阅该组织频道 -> 403 Forbidden
        let req_non_member = ChannelSubscriptionRequest { channel: org_channel };
        let res_non_member = request
            .post("/v1/realtime/subscription-token")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", non_member.token),
            )
            .json(&req_non_member)
            .await;

        res_non_member.assert_status(StatusCode::FORBIDDEN);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn cannot_subscribe_with_empty_channel_name() {
    request::<App, _, _>(|request, ctx| async move {
        let auth = super::prepare_data::init_user_login_unique(&request, &ctx).await;

        let req = ChannelSubscriptionRequest {
            channel: "".to_string(),
        };
        let res = request
            .post("/v1/realtime/subscription-token")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .json(&req)
            .await;

        res.assert_status(StatusCode::BAD_REQUEST);
    })
    .await;
}
