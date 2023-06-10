use opentelemetry::{
    sdk::{
        trace::{Config as TraceConfig, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
};
use std::{collections::HashMap, env, time::Duration};
use tracing_core::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt, Registry,
};
use url::Url;

use crate::Error;

const CLOUD_URL: &str = "https://cloud.axiom.co";

/// The guard will shutdown the tracer provider on drop.
#[must_use = "dropping the guard will shut down the tracer provider"]
#[derive(Debug)]
pub struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

/// Builder for creating a tracer, a layer or a subscriber that sends traces to
/// Axiom.
/// The token and the url are derived from the `AXIOM_TOKEN` and `AXIOM_URL`
/// environment variables.
#[derive(Debug, Default)]
pub struct Builder {
    dataset_name: Option<String>,
    token: Option<String>,
    url: Option<String>,
    trace_config: Option<TraceConfig>,
    service_name: Option<String>,
    no_env: bool,
}

impl Builder {
    /// Create a new Builder.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dataset(mut self, dataset_name: impl Into<String>) -> Self {
        self.dataset_name = Some(dataset_name.into());
        self
    }

    /// Set the Axiom API token to use.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the Axiom API URL to use. Defaults to Axiom Cloud.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the trace config.
    pub fn with_trace_config(mut self, trace_config: impl Into<TraceConfig>) -> Self {
        self.trace_config = Some(trace_config.into());
        self
    }

    /// Set the service name. It will be set as a resource attribute with the
    /// name `service_name`.
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    /// Don't fall back to environment variables.
    pub fn no_env(mut self) -> Self {
        self.no_env = true;
        self
    }

    /// Initialize the global subscriber. This panics if the initialization was
    /// unsuccessful, likely because a global subscriber was already installed or
    /// `AXIOM_TOKEN` is not set or invalid.
    pub fn init(self) -> Guard {
        self.try_init().unwrap()
    }

    /// Initialize the global subscriber. This returns an error if the
    /// initialization was unsuccessful, likely because a global subscriber was
    /// already installed or `AXIOM_TOKEN` is not set or invalid.
    pub fn try_init(self) -> Result<Guard, Error> {
        let (layer, guard) = self.layer()?;
        Registry::default().with(layer).try_init()?;
        Ok(guard)
    }

    /// Create a layer which sends traces to Axiom and a Guard which will shut
    /// down the tracer provider on drop.
    pub fn layer<S>(self) -> Result<(OpenTelemetryLayer<S, Tracer>, Guard), Error>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        let tracer = self.tracer()?;
        let layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Ok((layer, Guard {}))
    }

    fn tracer(self) -> Result<Tracer, Error> {
        let mut token = self.token;
        if !self.no_env {
            token = token.or_else(|| env::var("AXIOM_TOKEN").ok());
        }
        let token = token.ok_or(Error::MissingToken)?;
        if token.is_empty() {
            return Err(Error::EmptyToken);
        } else if !token.starts_with("xaat-") {
            return Err(Error::InvalidToken);
        }

        let mut dataset_name = self.dataset_name;
        if !self.no_env {
            dataset_name = dataset_name.or_else(|| env::var("AXIOM_DATASET").ok());
        }
        let dataset_name = dataset_name.ok_or(Error::MissingDatasetName)?;
        if dataset_name.is_empty() {
            return Err(Error::EmptyDatasetName);
        }

        let mut url = self.url;
        if !self.no_env {
            url = url.or_else(|| env::var("AXIOM_URL").ok());
        }
        let url = url
            .and_then(|url| if !url.is_empty() { Some(url) } else { None })
            .unwrap_or_else(|| CLOUD_URL.to_string())
            .parse::<Url>()?
            .join("/api/v1/traces")?;

        let mut headers = HashMap::with_capacity(2);
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert("X-Axiom-Dataset".to_string(), dataset_name);
        headers.insert(
            "User-Agent".to_string(),
            format!("tracing-axiom/{}", env!("CARGO_PKG_VERSION")),
        );

        let mut trace_config = self.trace_config.unwrap_or_default();
        if let Some(service_name) = self.service_name {
            trace_config = trace_config.with_resource(Resource::new(vec![KeyValue::new(
                SERVICE_NAME,
                service_name, // TODO: Is there a way to get the name of the bin crate using this?
            )]));
        }
        trace_config = trace_config.with_resource(Resource::new(vec![
            KeyValue::new(TELEMETRY_SDK_NAME, "tracing-axiom".to_string()),
            KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
            KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        ]));

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_http_client(reqwest::Client::new())
                    .with_endpoint(url)
                    .with_headers(headers)
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(trace_config)
            .install_batch(opentelemetry::runtime::Tokio)?;
        Ok(tracer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_token() {
        match Builder::new().no_env().try_init() {
            Err(Error::MissingToken) => {}
            result => panic!("expected MissingToken, got {:?}", result),
        };
    }

    #[test]
    fn test_empty_token() {
        match Builder::new().no_env().with_token("").try_init() {
            Err(Error::EmptyToken) => {}
            result => panic!("expected EmptyToken, got {:?}", result),
        };
    }

    #[test]
    fn test_invalid_token() {
        match Builder::new().no_env().with_token("invalid").try_init() {
            Err(Error::InvalidToken) => {}
            result => panic!("expected InvalidToken, got {:?}", result),
        };
    }

    #[test]
    fn test_invalid_url() {
        match Builder::new()
            .no_env()
            .with_token("xaat-123456789")
            .with_dataset("test")
            .with_url("<invalid>")
            .try_init()
        {
            Err(Error::InvalidUrl(_)) => {}
            result => panic!("expected InvalidUrl, got {:?}", result),
        };
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token() {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let result: Result<(OpenTelemetryLayer<Registry, Tracer>, Guard), Error> = Builder::new()
            .with_dataset("test")
            .with_token("xaat-123456789")
            .layer();
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token_env() {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let env_backup = env::var("AXIOM_TOKEN");
        env::set_var("AXIOM_TOKEN", "xaat-1234567890");

        let result: Result<(OpenTelemetryLayer<Registry, Tracer>, Guard), Error> =
            Builder::new().with_dataset("test").layer();

        if let Ok(token) = env_backup {
            env::set_var("AXIOM_TOKEN", token);
        }

        assert!(result.is_ok(), "{:?}", result.err());
    }
}
