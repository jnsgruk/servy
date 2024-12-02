mod common;
use common::{mock_redirect_source, servy};

use axum::http::{self, StatusCode};

#[tokio::test]
async fn test_serve_index_from_root() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/").await;

    response.assert_status(StatusCode::OK);
    response.assert_text("<body>Hello, World!</body>")
}

#[tokio::test]
async fn test_serve_index_from_directory() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/nested").await;

    response.assert_status(StatusCode::OK);
    response.assert_text("<body>Hello, Nested World!</body>")
}

#[tokio::test]
async fn test_serve_index_from_directory_trailing_slash() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/nested/").await;

    response.assert_status(StatusCode::OK);
    response.assert_text("<body>Hello, Nested World!</body>")
}

#[tokio::test]
async fn test_serve_file_by_path() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/css/main.css").await;

    response.assert_status(StatusCode::OK);
    response.assert_text("body { background-color: #fff; }")
}

#[tokio::test]
async fn test_serve_file_by_path_if_none_match() {
    let server = mock_redirect_source().await;
    let app = servy(format!("{}{}", server.url(), "/mock_redirects").as_str());

    let response = app.get("/css/main.css").await;
    let etag = response
        .header("ETag")
        .to_owned()
        .to_str()
        .unwrap()
        .to_string();

    let response = app
        .get("/css/main.css")
        .add_header(http::header::IF_NONE_MATCH, etag)
        .await;

    response.assert_status(StatusCode::NOT_MODIFIED);
    response.assert_text("")
}
