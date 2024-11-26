mod common;
use axum_test::TestServer;
use common::{mock_redirect_source, servy};

use axum::http::StatusCode;
use servy::metrics_router;

// For now all of these tests are in the same test because you can only have one global recorder.
// There is probably a way to solve this, but for now this serves as a good enough way to ensure
// that the metrics are functioning and the descriptions are set.
#[tokio::test]
async fn test_redirects_defined_metric() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let router = metrics_router().expect("failed to initialise servy metrics router");
    let metrics_app =
        TestServer::new(router).expect("failed to bootstrap servy metrics test server");

    let requests = vec!["/", "/foo", "/bar", "/index.html", "/ggggggggg"];

    for r in requests {
        app.get(r).await;
    }

    let metrics_response = metrics_app.get("/metrics").await;

    let expected_lines = vec![
        "# HELP servy_requests_total The total number of HTTP requests made to servy",
        "# TYPE servy_requests_total counter",
        "servy_requests_total 5",
        //
        "# HELP servy_response_status The status codes of HTTP responses",
        "# TYPE servy_response_status counter",
        "servy_response_status{status=\"308\"} 2",
        "servy_response_status{status=\"404\"} 1",
        "servy_response_status{status=\"200\"} 2",
        //
        "# HELP servy_redirects_served The number of requests per redirect",
        "# TYPE servy_redirects_served counter",
        "servy_redirects_served 0",
        "servy_redirects_served{alias=\"bar\"} 1",
        "servy_redirects_served{alias=\"foo\"} 1",
        //
        "# HELP servy_redirects_defined The number of redirects defined",
        "# TYPE servy_redirects_defined gauge",
        "servy_redirects_defined 2",
    ];

    for l in expected_lines {
        metrics_response.assert_text_contains(l);
    }

    // Ensure that the metrics endpoint responds
    metrics_response.assert_status(StatusCode::OK);
}
