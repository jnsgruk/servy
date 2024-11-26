use anyhow::{Context, Result};
use std::env;

const CONFIG_ENV_PREFIX: &str = "SERVY";

const CONFIG_REDIRECTS_URL: &str = "REDIRECTS_URL";
const CONFIG_LOG_LEVEL: &str = "LOG_LEVEL";
const CONFIG_HOST: &str = "HOST";
const CONFIG_PORT: &str = "PORT";
const CONFIG_METRICS_PORT: &str = "METRICS_PORT";

const DEFAULT_LOG_LEVEL: &str = "INFO";
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_METRICS_PORT: u16 = 8081;

/// Defines the configuration for Servy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub log_level: String,
    pub redirects_url: String,
    pub host: String,
    pub servy_port: u16,
    pub metrics_port: u16,
}

impl Config {
    /// Constructor for building a configuration by hand.
    pub fn new(
        host: &str,
        servy_port: u16,
        metrics_port: u16,
        log_level: &str,
        redirects_url: &str,
    ) -> Self {
        Config {
            host: host.to_string(),
            servy_port,
            metrics_port,
            log_level: log_level.to_string(),
            redirects_url: redirects_url.to_string(),
        }
    }
    /// Default configuration constructor.
    pub fn default_with_redirects(redirects_url: &str) -> Self {
        Config {
            host: String::from(DEFAULT_HOST),
            servy_port: DEFAULT_PORT,
            metrics_port: DEFAULT_METRICS_PORT,
            log_level: String::from(DEFAULT_LOG_LEVEL),
            redirects_url: redirects_url.to_string(),
        }
    }

    /// Loads the application's configuration from environment variables, with
    /// defaults where appropriate.
    pub fn from_env() -> Result<Config> {
        let redirects_url = load_env(CONFIG_REDIRECTS_URL)?;
        let log_level = load_env_or_default(CONFIG_LOG_LEVEL, DEFAULT_LOG_LEVEL);
        let host = load_env_or_default(CONFIG_HOST, DEFAULT_HOST);
        let servy_port: u16 = load_env_or_default(CONFIG_PORT, "8080").parse::<u16>()?;
        let metrics_port: u16 = load_env_or_default(CONFIG_METRICS_PORT, "8081").parse::<u16>()?;

        Ok(Config {
            redirects_url,
            log_level,
            host,
            servy_port,
            metrics_port,
        })
    }

    /// Return a [`String`] representing the socket to bind the app server to.
    pub fn servy_socket(&self) -> String {
        let Config {
            servy_port: port,
            host,
            ..
        } = self;
        format!("{host}:{port}")
    }

    /// Return a [`String`] representing the socket to bind the metrics server to.
    pub fn metrics_socket(&self) -> String {
        let Config {
            metrics_port: port,
            host,
            ..
        } = self;
        format!("{host}:{port}")
    }
}

/// Load a given environment variable.
fn load_env(key: &str) -> Result<String> {
    let key_with_prefix = format!("{CONFIG_ENV_PREFIX}_{key}");

    env::var(key_with_prefix).with_context(|| {
        format!(
            "failed to load environment variable {}_{}",
            CONFIG_ENV_PREFIX, key
        )
    })
}

/// Load a given environment variable or return a default if the environment variable is not set.
fn load_env_or_default(key: &str, default: &str) -> String {
    load_env(key).unwrap_or(default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn test_redirects_url() -> String {
        "http://example.com/redirects.json".to_string()
    }

    #[fixture]
    fn test_config(test_redirects_url: String) -> Config {
        Config::new("127.0.0.1", 8080, 8081, "INFO", &test_redirects_url)
    }

    #[rstest]
    fn test_new_config(test_redirects_url: String) {
        let config = Config::new("127.0.0.1", 8080, 8081, "INFO", &test_redirects_url);

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.servy_port, 8080);
        assert_eq!(config.metrics_port, 8081);
        assert_eq!(config.log_level, "INFO");
        assert_eq!(config.redirects_url, test_redirects_url);
    }

    #[rstest]
    fn test_default_with_redirects(test_redirects_url: String) {
        let config = Config::default_with_redirects(&test_redirects_url);

        assert_eq!(config.host, DEFAULT_HOST);
        assert_eq!(config.servy_port, DEFAULT_PORT);
        assert_eq!(config.metrics_port, DEFAULT_METRICS_PORT);
        assert_eq!(config.log_level, DEFAULT_LOG_LEVEL);
        assert_eq!(config.redirects_url, test_redirects_url);
    }

    #[rstest]
    #[case("127.0.0.1", 8080, "127.0.0.1:8080")]
    #[case("0.0.0.0", 9090, "0.0.0.0:9090")]
    fn test_servy_socket(#[case] host: &str, #[case] port: u16, #[case] expected: &str) {
        let config = Config::new(host, port, 8081, "INFO", "http://example.com");
        assert_eq!(config.servy_socket(), expected);
    }

    #[rstest]
    #[case("127.0.0.1", 8081, "127.0.0.1:8081")]
    #[case("0.0.0.0", 9091, "0.0.0.0:9091")]
    fn test_metrics_socket(#[case] host: &str, #[case] port: u16, #[case] expected: &str) {
        let config = Config::new(host, 8080, port, "INFO", "http://example.com");
        assert_eq!(config.metrics_socket(), expected);
    }

    #[rstest]
    fn test_load_env_or_default() {
        env::remove_var("SERVY_TEST_KEY");
        assert_eq!(load_env_or_default("TEST_KEY", "default"), "default");

        env::set_var("SERVY_TEST_KEY", "value");
        assert_eq!(load_env_or_default("TEST_KEY", "default"), "value");
        env::remove_var("SERVY_TEST_KEY");
    }
}
