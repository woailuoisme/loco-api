//! # OpenTelemetry HTTP 遥测与指标监控中间件
//!
//! 提供 HTTP 请求的 W3C TraceContext 提取、Span 生命周期关联、
//! 请求耗时与活跃请求数指标统计，并在响应头中回填 `x-trace-id` 与 `traceparent`。

use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use opentelemetry::{KeyValue, propagation::TextMapPropagator, trace::TraceContextExt};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::common::telemetry::{HTTP_METRICS, HeaderExtractor, HeaderInjector};

/// HTTP 遥测与指标跟踪中间件
///
/// 通过 Axum 提取器直接捕获 `Option<MatchedPath>`：
/// 1. 若命中实际路由模板，采用模板路径（例如 `/v1/users/:id`）；
/// 2. 若未命中任何路由（404 Fallback），统一规约为 `"not_found"`，彻底防御高基数内存爆炸。
pub async fn telemetry_middleware(matched_path: Option<MatchedPath>, req: Request, next: Next) -> Response {
    let method = req.method().to_string();

    // 路由模板规范化 (有效匹配采用模板，未匹配一律归约至 "not_found")
    let route = matched_path
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "not_found".to_string());

    // 1. 提取上游传入的 W3C TraceContext (traceparent / tracestate)
    let propagator = TraceContextPropagator::new();
    let parent_cx = propagator.extract(&HeaderExtractor(req.headers()));

    // 确定上游提取的 TraceId (若存在)
    let fallback_trace_id = {
        let span_ref = parent_cx.span();
        let sc = span_ref.span_context();
        if sc.is_valid() { Some(sc.trace_id()) } else { None }
    };

    // 2. 创建当前请求的 Tracing Span 并关联父级 Context
    let span = tracing::info_span!(
        "HTTP request",
        http.method = %method,
        http.route = %route,
        http.status_code = tracing::field::Empty,
        otel.kind = "SERVER",
    );
    let _ = span.set_parent(parent_cx.clone());

    // 3. 递增活跃请求数
    if let Some(metrics) = HTTP_METRICS.get() {
        metrics.active_requests.add(
            1,
            &[
                KeyValue::new("http.method", method.clone()),
                KeyValue::new("http.route", route.clone()),
            ],
        );
    }

    let start_time = Instant::now();

    // 4. 执行后续中间件与业务 Handler (在 span 作用域内)
    let mut response = next.run(req).instrument(span.clone()).await;

    let elapsed = start_time.elapsed().as_secs_f64();
    let status_code = response.status().as_u16();

    // 5. 记录 Span 状态
    span.record("http.status_code", status_code);

    // 6. 统计指标与直方图
    if let Some(metrics) = HTTP_METRICS.get() {
        let attrs = [
            KeyValue::new("http.method", method),
            KeyValue::new("http.route", route),
            KeyValue::new("http.status_code", status_code as i64),
        ];

        metrics.duration.record(elapsed, &attrs);
        metrics.total_requests.add(1, &attrs);
        metrics.active_requests.add(-1, &[attrs[0].clone(), attrs[1].clone()]);
    }

    // 7. 回填 x-trace-id 与 traceparent 响应头
    let otel_cx = span.context();
    let otel_span_ref = otel_cx.span();
    let otel_sc = otel_span_ref.span_context();

    // 优先采用活动 OTel Context，次选上游链路，若皆无则生成合法 128 位随机 Trace ID
    let trace_id = if otel_sc.is_valid() {
        otel_sc.trace_id()
    } else if let Some(fid) = fallback_trace_id {
        fid
    } else {
        opentelemetry::trace::TraceId::from(rand::random::<u128>())
    };

    let trace_id_hex = format!("{:032x}", trace_id);
    if let Ok(val) = axum::http::HeaderValue::from_str(&trace_id_hex) {
        response.headers_mut().insert("x-trace-id", val);
    }

    // 若有活动 OTel 上下文或上游 W3C 上下文，注入标准 traceparent；否则回填基础 traceparent
    if otel_sc.is_valid() {
        propagator.inject_context(&otel_cx, &mut HeaderInjector(response.headers_mut()));
    } else if parent_cx.span().span_context().is_valid() {
        propagator.inject_context(&parent_cx, &mut HeaderInjector(response.headers_mut()));
    } else if response.headers().get("traceparent").is_none() {
        let traceparent_val = format!("00-{trace_id_hex}-0000000000000001-01");
        if let Ok(val) = axum::http::HeaderValue::from_str(&traceparent_val) {
            response.headers_mut().insert("traceparent", val);
        }
    }

    response
}
