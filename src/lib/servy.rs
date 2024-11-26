use anyhow::{Context, Result};
use axum::{
    error_handling::HandleErrorLayer,
    extract::Request,
    middleware::{self},
    response::Response,
    routing::get,
    Router,
};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{future::ready, net::SocketAddr, time::Duration};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use tower::ServiceBuilder;
use tracing::{info, info_span, Span};

use crate::{
    handlers::{default_handler, error_handler, root_handler},
    metrics::{init_metrics, metrics_middleware},
    AppContext, Config,
};

/// Start the app server and metrics servers in separate threads.
pub async fn run(config: Config) -> Result<()> {
    tokio::try_join!(start_app_server(&config), start_metrics_server(&config))?;
    Ok(())
}

/// Start the Servy app server according to the given configuration.
async fn start_app_server(config: &Config) -> Result<()> {
    let socket: SocketAddr = config.servy_socket().parse()?;

    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .with_context(|| format!("failed to start app server listener on: {}", socket))?;

    let context = AppContext::new(config.clone());
    context.refresh_redirects().await?;

    let app = servy_router(context)?;

    info!("starting servy server on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("error running app server")?;

    Ok(())
}

/// Start the Servy metrics server according to the given configuration.
async fn start_metrics_server(config: &Config) -> Result<()> {
    let socket: SocketAddr = config.metrics_socket().parse()?;

    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .with_context(|| format!("failed to start metrics server listener on: {}", socket))?;

    let app = metrics_router()?;

    info!("starting metrics server on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .context("error running metrics server")?;

    Ok(())
}

/// Construct and return an Axum router which services requests to the `/metrics` endpoint
/// for the metrics server.
pub fn metrics_router() -> Result<Router> {
    let prometheus_handle = PrometheusBuilder::new().install_recorder()?;
    let router = Router::new().route("/metrics", get(move || ready(prometheus_handle.render())));

    // Initialise metrics with the correct types and descriptions.
    init_metrics();

    Ok(router)
}

/// Construct and return an Axum router for the main Servy app which includes configuration
/// for tracing, load shedding and compression.
pub fn servy_router(context: AppContext) -> Result<Router> {
    let router = Router::new()
        .route("/*key", get(default_handler))
        .route("/", get(root_handler))
        .route_layer(middleware::from_fn(metrics_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(HandleErrorLayer::new(error_handler))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(make_span)
                        .on_response(trace_on_response),
                )
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10)),
        )
        .with_state(context);

    Ok(router)
}

/// Construct a span which includes placeholders for the fields that could be populated through the
/// duration of a request/response cycle in Servy.
fn make_span<T>(request: &Request<T>) -> Span {
    let span = info_span!(
        "servy_request",
        "request.method" = %request.method(),
        "request.uri" = %request.uri(),
        "request.user_agent" = tracing::field::Empty,
        "response.status_code" = tracing::field::Empty,
        "response.file" = tracing::field::Empty,
        "response.location" = tracing::field::Empty,
    );

    // If we can determine the user agent of the request, record it in the span now.
    if let Some(h) = request.headers().get("user-agent") {
        if let Ok(s) = h.to_str() {
            span.record("request.user_agent", s);
        }
    }

    span
}

/// Ensure the status code for every response is logged in the current span and log the
/// span at "INFO" level.
fn trace_on_response(response: &Response, _latency: Duration, span: &Span) {
    span.record(
        "response.status_code",
        tracing::field::display(response.status().as_u16()),
    );
    tracing::info!("served request");
}
