use loco_api::app::App;
use loco_rs::testing::prelude::*;

#[tokio::test]
#[serial_test::serial]
async fn test_request_with_traceparent_propagates_trace_id() {
    request::<App, _, _>(|request, _ctx| async move {
        let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
        let traceparent = format!("00-{trace_id}-00f067aa0ba902b7-01");

        let res = request.get("/").add_header("traceparent", &traceparent).await;

        assert_eq!(res.status_code(), 200);

        let headers = res.headers();
        let res_trace_id = headers.get("x-trace-id").and_then(|v| v.to_str().ok());
        assert_eq!(res_trace_id, Some(trace_id));

        let res_traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
        assert!(res_traceparent.is_some());
        assert!(res_traceparent.unwrap().contains(trace_id));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_telemetry_not_found_route_includes_headers() {
    request::<App, _, _>(|request, _ctx| async move {
        let trace_id = "11223344556677889900aabbccddeeff";
        let traceparent = format!("00-{trace_id}-1234567890abcdef-01");

        let res = request
            .get("/non-existent-path")
            .add_header("traceparent", &traceparent)
            .await;

        assert_eq!(res.status_code(), 404);

        let headers = res.headers();
        let res_trace_id = headers.get("x-trace-id").and_then(|v| v.to_str().ok());
        assert_eq!(res_trace_id, Some(trace_id));
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn test_request_without_traceparent_generates_and_injects_trace_id() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/").await;
        assert_eq!(res.status_code(), 200);

        let headers = res.headers();
        let res_trace_id = headers.get("x-trace-id").and_then(|v| v.to_str().ok());
        assert!(
            res_trace_id.is_some(),
            "x-trace-id header must be present on all responses"
        );
        let trace_id = res_trace_id.unwrap();
        assert_eq!(trace_id.len(), 32, "x-trace-id must be 32 hex chars");
        assert!(
            trace_id.chars().all(|c| c.is_ascii_hexdigit()),
            "x-trace-id must be valid hex"
        );

        let res_traceparent = headers.get("traceparent").and_then(|v| v.to_str().ok());
        assert!(res_traceparent.is_some(), "traceparent header must be present");
        assert!(res_traceparent.unwrap().contains(trace_id));
    })
    .await;
}
