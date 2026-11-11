use axum::http::{HeaderMap, HeaderValue};
use loco_api::common::telemetry::{HeaderExtractor, HeaderInjector, OtelConfig};
use opentelemetry::{
    propagation::{Extractor, Injector, TextMapPropagator},
    trace::{TraceContextExt, TraceId},
};
use opentelemetry_sdk::propagation::TraceContextPropagator;

#[test]
fn test_otel_config_defaults() {
    let config = OtelConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.endpoint, "http://127.0.0.1:4318");
    assert_eq!(config.service_name, "loco-api");
    assert!((config.sample_ratio - 1.0).abs() < f64::EPSILON);
    assert_eq!(config.export_timeout_secs, 3);
}

#[test]
fn test_otel_config_deserialize() {
    let json = serde_json::json!({
        "enabled": true,
        "endpoint": "http://otel-collector:4318",
        "service_name": "loco-api-test",
        "sample_ratio": 0.5,
        "export_timeout_secs": 10
    });

    let config: OtelConfig = serde_json::from_value(json).expect("Failed to deserialize OtelConfig");
    assert!(config.enabled);
    assert_eq!(config.endpoint, "http://otel-collector:4318");
    assert_eq!(config.service_name, "loco-api-test");
    assert!((config.sample_ratio - 0.5).abs() < f64::EPSILON);
    assert_eq!(config.export_timeout_secs, 10);
}

#[test]
fn test_header_extractor_and_injector() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "traceparent",
        HeaderValue::from_static("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    );

    let extractor = HeaderExtractor(&headers);
    assert_eq!(
        extractor.get("traceparent"),
        Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
    );
    assert!(extractor.keys().contains(&"traceparent"));

    let mut out_headers = HeaderMap::new();
    let mut injector = HeaderInjector(&mut out_headers);
    injector.set("custom-trace", "test-val".to_string());
    assert_eq!(
        out_headers.get("custom-trace").and_then(|v| v.to_str().ok()),
        Some("test-val")
    );
}

#[test]
fn test_w3c_trace_context_propagation() {
    let propagator = TraceContextPropagator::new();

    let mut req_headers = HeaderMap::new();
    let raw_traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    req_headers.insert("traceparent", HeaderValue::from_static(raw_traceparent));

    // 提取 Context
    let cx = propagator.extract(&HeaderExtractor(&req_headers));
    let span_ref = cx.span();
    let span_ctx = span_ref.span_context();
    assert!(span_ctx.is_valid());
    let expected_trace_id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
    assert_eq!(span_ctx.trace_id(), expected_trace_id);

    // 重新注入响应头
    let mut res_headers = HeaderMap::new();
    propagator.inject_context(&cx, &mut HeaderInjector(&mut res_headers));
    let injected_traceparent = res_headers.get("traceparent").and_then(|v| v.to_str().ok()).unwrap();
    assert!(injected_traceparent.starts_with("00-4bf92f3577b34da6a3ce929d0e0e4736-"));
}

#[test]
fn test_http_metrics_recording() {
    let meter = opentelemetry::global::meter("test-metrics");
    let metrics = loco_api::common::telemetry::HttpMetrics::new(&meter);

    let attrs = [
        opentelemetry::KeyValue::new("http.method", "GET"),
        opentelemetry::KeyValue::new("http.route", "/v1/users/:id"),
        opentelemetry::KeyValue::new("http.status_code", 200i64),
    ];

    metrics.active_requests.add(1, &attrs[0..2]);
    metrics.duration.record(0.042, &attrs);
    metrics.total_requests.add(1, &attrs);
    metrics.active_requests.add(-1, &attrs[0..2]);
}
