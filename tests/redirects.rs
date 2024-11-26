mod common;
use common::{mock_redirect_source, servy};

use axum::http::StatusCode;

#[tokio::test]
async fn test_bad_redirect_source_error() {
    let app = servy("this-is-a-bad-url");
    let response = app.get("/foo").await;
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_known_redirect() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/foo").await;

    response.assert_status(StatusCode::PERMANENT_REDIRECT);
    response.assert_header("location", "http://foo.bar");
}

#[tokio::test]
async fn test_known_redirect_trailing_slash() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/foo/").await;

    response.assert_status(StatusCode::PERMANENT_REDIRECT);
    response.assert_header("location", "http://foo.bar");
}

#[tokio::test]
async fn test_unknown_redirect() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/unknown-redirect").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_unknown_redirect_solved_with_refresh() {
    let mut server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/baz").await;
    response.assert_status(StatusCode::NOT_FOUND);

    // Now repopulate the redirects in the upstream source with a new map.
    server
        .mock("GET", "/mock_redirects")
        .match_query(mockito::Matcher::Regex("cachebust=[0-9]+".into()))
        .with_body("baz http://baz.qux")
        .create_async()
        .await;

    let response = app.get("/baz").await;
    response.assert_status(StatusCode::PERMANENT_REDIRECT);
    response.assert_header("location", "http://baz.qux");
}
