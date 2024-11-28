use crate::{metrics::REDIRECTS_SERVED, AppContext};
use anyhow::{bail, Error, Result};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{self, HeaderMap, Request, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_embed::ServeEmbed;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use tower::{BoxError, ServiceExt};
use tracing::Span;

#[derive(RustEmbed, Clone)]
#[folder = "$SERVY_ASSETS_DIR"]
struct Assets;

/// Handle requests to the root URL "/" - delegating to the default_handler.
pub async fn root_handler(
    headers: HeaderMap,
    State(context): State<AppContext>,
) -> impl IntoResponse {
    default_handler(Path("/".to_string()), State(context), headers).await
}

/// Default non-root path handler which attempts to first match a filename, then a redirect.
pub async fn default_handler(
    Path(path): Path<String>,
    State(context): State<AppContext>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match handle_file(&headers, &path).await {
        Ok(file) => file.into_response(),
        Err(_) => match handle_redirect(&path, &context).await {
            Ok(redirect) => redirect.into_response(),
            Err(_) => handle_not_found(&headers).await,
        },
    }
}

/// Handle load-shedding and timeout errors with the appropriate status codes.
pub async fn error_handler(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {error}")),
    )
}

/// Construct a 308 Permanent Redirect if the given redirect is specified. If the initial lookup
/// fails, then refresh the redirects map and try again. If the redirect is still not specified
/// then return an error.
async fn handle_redirect(path: &str, context: &AppContext) -> Result<Response> {
    let redirects = context.redirects();
    let key = path.strip_suffix("/").unwrap_or(path).to_string();

    if let Some(value) = redirects.get(&key) {
        do_redirect(&key, value)
    } else {
        let redirects = context.refresh_redirects().await?;

        if let Some(value) = redirects.get(&key) {
            do_redirect(&key, value)
        } else {
            Err(Error::msg("no redirect found for key"))
        }
    }
}

/// Construct a response for a given filepath. Use the embedded file server to return the
/// appropriate file, recording the filename in the current span. If the path specified is a
/// directory, and the directory contains an 'index.html' file, then serve it.
async fn handle_file(headers: &HeaderMap, path: &str) -> Result<Response> {
    let file_server = ServeEmbed::<Assets>::new();

    let mut filename = path.to_string();
    if filename.is_empty() || filename == "/" {
        filename = "index.html".to_string();
    } else {
        let idx_file = format!("{}/index.html", &filename);
        if Assets::get(&idx_file).is_some() {
            filename = idx_file;
        }
    }

    let req = Request::builder()
        .uri(format!("/{}", filename.clone()))
        .body(Body::empty())?;

    let mut resp = file_server.oneshot(req).await?;

    // Check the If-None-Match header of the request
    let request_inm = header_val_or_empty(http::header::IF_NONE_MATCH, headers);

    if !request_inm.is_empty() {
        // Get the ETag header of the potential response from ServeEmbed
        let response_etag = header_val_or_empty(http::header::ETAG, resp.headers());

        // If the ETag of the response matches the If-None-Match header of the request,
        // return a 304 Not Modified response
        if request_inm == response_etag {
            return Ok(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())
                .unwrap_or(
                    (StatusCode::NOT_MODIFIED, String::from("Not Modified")).into_response(),
                ));
        }
    }

    if resp.status().is_success() {
        if filename == "404.html" {
            let status = resp.status_mut();
            *status = StatusCode::NOT_FOUND;
        }

        Span::current().record("response.file", filename.clone());
        return Ok(resp);
    }

    bail!("file not found: {}", filename.clone())
}

/// Construct an appropriate "Not Found" response. If there is a 404.html page present
/// in the webroot, then serve that, otherwise serve a plain response with a 404 status code
async fn handle_not_found(headers: &HeaderMap) -> Response<Body> {
    match handle_file(headers, "404.html").await {
        Ok(response) => response,
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found".to_string()))
            .unwrap_or((StatusCode::NOT_FOUND, String::from("Not found")).into_response()),
    }
}

/// Construct and return a permanent redirect response for a given specified redirect.
/// Ensure that the relevant metrics and spans are updated.
fn do_redirect(key: &str, redirect: &str) -> Result<Response> {
    Span::current().record("response.location", redirect);

    let labels = [("alias", key.to_string())];
    metrics::counter!(REDIRECTS_SERVED.namespaced_name(), &labels).increment(1);

    Ok(Redirect::permanent(redirect).into_response())
}

/// Extract a header value from a header map, or return an empty string if the header is absent.
fn header_val_or_empty(header: http::HeaderName, headers: &HeaderMap) -> String {
    headers
        .get(header)
        .and_then(|value| value.to_str().ok().map(|value| value.to_string()))
        .unwrap_or("".to_string())
}

#[cfg(test)]
mod tests {
    use http::HeaderName;

    use super::*;

    #[test]
    fn test_header_val_or_empty() {
        let mut headers = HeaderMap::new();
        let header_name = HeaderName::from_static("x-test-header");

        // Test when header is present
        headers.insert(header_name.clone(), "test_value".parse().unwrap());
        assert_eq!(
            header_val_or_empty(header_name.clone(), &headers),
            "test_value"
        );

        // Test when header is absent
        let absent_header_name = HeaderName::from_static("x-absent-header");
        assert_eq!(header_val_or_empty(absent_header_name, &headers), "");
    }
}
