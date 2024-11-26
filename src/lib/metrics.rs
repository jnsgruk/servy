use axum::{extract::Request, middleware::Next, response::IntoResponse};

/// A struct representing a given metric for Servy.
#[derive(PartialEq, Debug)]
pub struct Metric {
    name: &'static str,
    namespace: &'static str,
    description: &'static str,
}

impl Metric {
    /// Returns the name of the metric, prefixed with the metric's namespace.
    pub fn namespaced_name(&self) -> String {
        format!("{}_{}", self.namespace, self.name)
    }
}

/// A counter metric representing the total number of HTTP requests made to servy.
pub const REQUESTS_TOTAL: Metric = Metric {
    name: "requests_total",
    namespace: "servy",
    description: "The total number of HTTP requests made to servy",
};

/// A counter metric representing the number of requests made to each redirect.
pub const REDIRECTS_SERVED: Metric = Metric {
    name: "redirects_served",
    namespace: "servy",
    description: "The number of requests per redirect",
};

/// A gauge metric that represents the number of redirects defined at a given time.
pub const REDIRECTS_DEFINED: Metric = Metric {
    name: "redirects_defined",
    namespace: "servy",
    description: "The number of redirects defined",
};

/// A counter metric keeping track of the number of HTTP responses by status code.
pub const RESPONSE_STATUS: Metric = Metric {
    name: "response_status",
    namespace: "servy",
    description: "The status codes of HTTP responses",
};

/// An array of gauge metrics.
const GAUGES: [Metric; 1] = [REDIRECTS_DEFINED];

/// An array of counter metrics.
const COUNTERS: [Metric; 3] = [REQUESTS_TOTAL, REDIRECTS_SERVED, RESPONSE_STATUS];

/// Iterate over each of the defined metrics ensuring they are initialized and described
/// per the definitions above.
pub fn init_metrics() {
    for metric in COUNTERS {
        metrics::describe_counter!(metric.namespaced_name(), metric.description);
        let _counter = metrics::counter!(metric.namespaced_name());
    }

    for metric in GAUGES {
        metrics::describe_gauge!(metric.namespaced_name(), metric.description);
        let _gauge = metrics::gauge!(metric.namespaced_name());
    }
}

/// An Axum middleware used to increment those metrics that report on global details about
/// requests made to Servy.
pub async fn metrics_middleware(req: Request, next: Next) -> impl IntoResponse {
    let response = next.run(req).await;

    let labels = [("status", response.status().as_u16().to_string())];
    metrics::counter!(RESPONSE_STATUS.namespaced_name(), &labels).increment(1);
    metrics::counter!(REQUESTS_TOTAL.namespaced_name()).increment(1);

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("test_metric", "test_ns", "test_desc", "test_ns_test_metric")]
    #[case("requests", "api", "API requests", "api_requests")]
    fn test_namespaced_name(
        #[case] name: &'static str,
        #[case] namespace: &'static str,
        #[case] description: &'static str,
        #[case] expected: &str,
    ) {
        let metric = Metric {
            name,
            namespace,
            description,
        };
        assert_eq!(metric.namespaced_name(), expected);
    }

    #[rstest]
    fn test_init_metrics() {
        init_metrics();
    }
}
