//! # 系统健康检查控制器 (Health Check Controller)
//!
//! 提供 Kubernetes Liveness 存活探针与 Readiness 就绪探针接口。

use axum::http::StatusCode;
use loco_rs::prelude::*;

pub use crate::common::diagnostics::{ComponentHealth, DiagnosticsHub, HealthChecks, ReadinessData};

/// Liveness 存活探针
///
/// 基础设施探针，不纳入 OpenAPI 对外文档。
/// 返回 JSON `{"status": "ok"}`。
#[debug_handler]
pub async fn liveness() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

/// Readiness 就绪探针
///
/// 基础设施探针，不纳入 OpenAPI 对外文档。
/// 无论就绪与否均返回同一 [`ReadinessData`] 根结构，仅以 HTTP 状态码区分
/// (`200` 就绪 / `503` 不可用)，符合 `api.md`「根结构可预测」要求。
#[debug_handler]
pub async fn readiness(State(ctx): State<AppContext>) -> Response {
    let data = DiagnosticsHub::check_readiness(&ctx).await;

    let status = if data.is_all_up() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(data)).into_response()
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/livez", get(liveness))
        .add("/readyz", get(readiness))
}
