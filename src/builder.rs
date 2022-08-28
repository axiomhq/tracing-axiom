use opentelemetry::{
    sdk::{
        trace::{Config as TraceConfig, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
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
#[must_use]
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
    token: Option<String>,
    url: Option<String>,
    trace_config: Option<TraceConfig>,
    service_name: Option<String>,
}

impl Builder {
    /// Create a new Builder.
    pub fn new() -> Self {
        Self::default()
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
        let token = self
            .token
            .ok_or(Error::MissingToken)
            .or_else(|_| env::var("AXIOM_TOKEN"))?;
        if token.is_empty() {
            return Err(Error::EmptyToken);
        } else if !token.starts_with("xaat-") {
            return Err(Error::InvalidToken);
        }

        let url =
            self.url
                .or_else(|| {
                    env::var("AXIOM_URL").ok().and_then(|url| {
                        if !url.is_empty() {
                            Some(url)
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| CLOUD_URL.to_string())
                .parse::<Url>()?
                .join("/api/v1/traces")?;

        let mut headers = HashMap::with_capacity(2);
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert(
            "User-Agent".to_string(),
            format!("tracing-axiom/{}", env!("CARGO_PKG_VERSION")),
        );

        let mut trace_config = self.trace_config.unwrap_or_default();
        if let Some(service_name) = self.service_name {
            trace_config = trace_config.with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                service_name, // can we be smarter about this?
            )]));
        }

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
