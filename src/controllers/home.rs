use crate::common::consts::openapi::TAG_HOME;
use loco_rs::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeResource {
    /// 服务运行状态
    pub status: String,
}

/// 根路由信息
#[utoipa::path(
    get,
    path = "/",
    summary = "获取根路由信息",
    description = "应用根路由状态",
    responses(
        (status = 200, description = "获取成功", body = HomeResource)
    ),
    tag = TAG_HOME
)]
#[debug_handler]
pub async fn index() -> Result<Response> {
    crate::dtos::common::format_res::ok(HomeResource {
        status: "ok".to_string(),
    })
}

#[debug_handler]
pub async fn favicon() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index)).add("/favicon.ico", get(favicon))
}
