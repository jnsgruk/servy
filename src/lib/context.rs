use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{redirects::parse_redirects, Config, Redirects};
use anyhow::Result;
use reqwest::Client;

#[derive(Clone, Debug)]

/// AppContext holds the context for a running Servy server, including the list of defined
/// redirects and a common HTTP client to be used across redirect map refreshes.
pub struct AppContext {
    redirects_url: String,
    redirects: Arc<RwLock<Redirects>>,
    http_client: Client,
}

impl AppContext {
    /// Construct a new AppContext for a given Servy configuration.
    pub fn new(config: Config) -> Self {
        Self {
            redirects_url: config.redirects_url,
            redirects: Arc::new(RwLock::new(Redirects::new())),
            http_client: Client::new(),
        }
    }

    /// Return the currently defined redirects.
    pub fn redirects(&self) -> Redirects {
        self.redirects.read().unwrap().clone()
    }

    /// Return the URL used to fetch/refresh the redirects map.
    pub fn redirects_url(&self) -> &str {
        &self.redirects_url
    }

    /// Refresh the redirects using the URL specified in the config.
    pub async fn refresh_redirects(&self) -> Result<Redirects> {
        let redirects = Self::fetch_redirects(self.redirects_url(), &self.http_client).await?;
        match self.redirects.write() {
            Ok(mut redirects_guard) => redirects_guard.clone_from(&redirects),
            Err(poisoned) => poisoned.into_inner().clone_from(&redirects),
        }
        metrics::gauge!("servy_redirects_defined").set(redirects.len() as f64);
        Ok(redirects)
    }

    /// Fetch the list of redirects from the defined upstream.
    async fn fetch_redirects(url: &str, client: &Client) -> Result<Redirects> {
        tracing::info!("fetching redirects from url: {url}");

        // Append a query parameter with current unix timestamp to break caching as required.
        let url = format!(
            "{url}?cachebust={}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs()
        );

        let resp = client.get(url).send().await?.text().await?;
        let map = parse_redirects(&resp);
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use mockito::{Server, ServerGuard};

    async fn mock_redirect_server() -> ServerGuard {
        let mut server = Server::new_async().await;

        server
            .mock("GET", "/mock_redirects")
            .match_query(mockito::Matcher::Regex("cachebust=[0-9]+".into()))
            .with_body("foo http://foo.bar\nbar http://bar.baz")
            .create_async()
            .await;

        server
    }

    #[test]
    fn test_redirects_url() {
        let config = Config::default_with_redirects("http://example.com");
        let context = AppContext::new(config);
        assert_eq!(context.redirects_url(), "http://example.com");
    }

    #[tokio::test]
    async fn test_fetch_redirects() {
        let client = Client::new();
        let server = mock_redirect_server().await;
        let url = format!("{}{}", server.url(), "/mock_redirects");

        let redirects = AppContext::fetch_redirects(&url, &client).await.unwrap();

        assert_eq!(redirects.len(), 2);
        assert_eq!(redirects.get("foo").unwrap(), "http://foo.bar");
        assert_eq!(redirects.get("bar").unwrap(), "http://bar.baz");
    }

    #[tokio::test]
    async fn test_refresh_redirects() {
        let server = mock_redirect_server().await;
        let url = format!("{}{}", server.url(), "/mock_redirects");

        let context = AppContext::new(Config::default_with_redirects(&url));
        context.refresh_redirects().await.unwrap();

        let redirects = context.redirects();

        assert_eq!(redirects.len(), 2);
        assert_eq!(redirects.get("foo").unwrap(), "http://foo.bar");
    }
}
