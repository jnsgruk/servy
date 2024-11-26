use axum_test::TestServer;
use mockito::{Server, ServerGuard};
use servy::{servy_router, AppContext, Config};

pub async fn mock_redirect_source() -> ServerGuard {
    let mut server = Server::new_async().await;

    let redirects = [
        "foo http://foo.bar",
        "# Comment",
        "bar http://bar.baz",
        "garbagethatshouldntbeparsed",
        "another line that shouldn't be parsed",
    ];

    server
        .mock("GET", "/mock_redirects")
        .match_query(mockito::Matcher::Regex("cachebust=[0-9]+".into()))
        .with_body(redirects.join("\n"))
        .create_async()
        .await;

    server
}

pub fn servy(redirect_source: &str) -> TestServer {
    let config = Config::default_with_redirects(redirect_source);
    let ctx = AppContext::new(config);
    let router = servy_router(ctx).expect("failed to initialise servy router");
    TestServer::new(router).expect("failed to bootstrap servy test server")
}
