use crate::Error;
use opentelemetry::{Key, KeyValue, Value};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{Config as TraceConfig, Tracer},
    Resource,
};
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME, TELEMETRY_SDK_VERSION,
};
use reqwest::Url;
use std::{
    collections::HashMap,
    env::{self, VarError},
    time::Duration,
};
use tracing_core::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

const CLOUD_URL: &str = "https://api.axiom.co";

/// Builder for creating a tracing tracer, a layer or a subscriber that sends traces to
/// Axiom via the `OpenTelemetry` protocol. The API token is read from the `AXIOM_TOKEN`
/// environment variable. The dataset name is read from the `AXIOM_DATASET` environment
/// variable. The URL defaults to Axiom Cloud whose URL is `https://cloud.axiom.co` but
/// can be overridden by setting the `AXIOM_URL` environment variable for testing purposes
///
#[derive(Debug, Default)]
pub struct Builder {
    dataset_name: Option<String>,
    token: Option<String>,
    url: Option<Url>,
    tags: Vec<KeyValue>,
    trace_config: Option<TraceConfig>,
    service_name: Option<String>,
    timeout: Option<Duration>,
}

fn get_env(env_var_name: &'static str) -> Result<Option<String>, Error> {
    match env::var(env_var_name) {
        Ok(maybe_ok_var) => Ok(Some(maybe_ok_var)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(Error::EnvVarNotUnicode(env_var_name.to_string())),
    }
}

impl Builder {
    /// Set the Axiom dataset name to use. The dataset name is the name of the
    /// persistent dataset in Axiom cloud that will store the traces and make
    /// them available for querying using APL, the Axiom SDK or the Axiom CLI.
    ///
    /// # Errors
    /// If the dataset name is empty.
    pub fn with_dataset(mut self, dataset_name: impl Into<String>) -> Result<Self, Error> {
        let dataset_name: String = dataset_name.into();
        if dataset_name.is_empty() {
            Err(Error::EmptyDataset)
        } else {
            self.dataset_name = Some(dataset_name);
            Ok(self)
        }
    }

    /// Set the Axiom API token to use.
    ///
    /// # Errors
    /// If the token is empty or does not start with `xaat-` (aka is not a api token).
    pub fn with_token(mut self, token: impl Into<String>) -> Result<Self, Error> {
        let token: String = token.into();
        if token.is_empty() {
            Err(Error::EmptyToken)
        } else if !token.starts_with("xaat-") {
            Err(Error::InvalidToken)
        } else {
            self.token = Some(token);
            Ok(self)
        }
    }

    /// Set the Axiom API URL to use. Defaults to Axiom Cloud. When not set Axiom Cloud is used.
    ///
    /// # Errors
    /// If the URL is not a valid URL.
    pub fn with_url(mut self, url: &str) -> Result<Self, Error> {
        self.url = Some(url.parse()?);
        Ok(self)
    }

    /// Set the trace config.
    #[must_use]
    pub fn with_trace_config(mut self, trace_config: impl Into<TraceConfig>) -> Self {
        self.trace_config = Some(trace_config.into());
        self
    }

    /// Set the service name. It will be set as a resource attribute with the
    /// name `service_name`.
    #[must_use]
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    /// Set the resource tags for the open telemetry tracer that publishes to Axiom.
    /// These tags will be added to all spans.
    #[must_use]
    pub fn with_tags<T, K, V>(mut self, tags: T) -> Self
    where
        K: Into<Key>,
        V: Into<Value>,
        T: Iterator<Item = (K, V)>,
    {
        self.tags = tags.map(|(k, v)| KeyValue::new(k, v)).collect::<Vec<_>>();
        self
    }

    /// Sets the collector timeout for the OTLP exporter.
    /// The default is 3 seconds.
    ///
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Load defaults from environment variables, if variables were set before this call they will not be replaced.
    ///
    /// The following environment variables are used:
    /// - `AXIOM_TOKEN`
    /// - `AXIOM_DATASET`
    /// - `AXIOM_URL`
    ///
    /// # Errors
    /// If an environment variable is not valid UTF8, or any of their values are invalid.
    pub fn with_env(mut self) -> Result<Self, Error> {
        if self.token.is_none() {
            if let Some(t) = get_env("AXIOM_TOKEN")? {
                self = self.with_token(t)?;
            }
        };

        if self.dataset_name.is_none() {
            if let Some(d) = get_env("AXIOM_DATASET")? {
                self = self.with_dataset(d)?;
            }
        };
        if self.url.is_none() {
            if let Some(u) = get_env("AXIOM_URL")? {
                self = self.with_url(&u)?;
            }
        };

        Ok(self)
    }

    /// Create a layer which sends traces to Axiom that can be added to the tracing layers.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the settings are not valid
    pub fn build<S>(self) -> Result<OpenTelemetryLayer<S, Tracer>, Error>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        Ok(tracing_opentelemetry::layer().with_tracer(self.tracer()?))
    }

    fn tracer(self) -> Result<Tracer, Error> {
        let token = self.token.ok_or(Error::MissingToken)?;
        let dataset_name = self.dataset_name.ok_or(Error::MissingDataset)?;
        let url = self
            .url
            .unwrap_or_else(|| CLOUD_URL.to_string().parse().expect("this is a valid URL"));

        let mut headers = HashMap::with_capacity(2);
        headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        headers.insert("X-Axiom-Dataset".to_string(), dataset_name);
        headers.insert(
            "User-Agent".to_string(),
            format!("tracing-axiom/{}", env!("CARGO_PKG_VERSION")),
        );

        let mut tags = self.tags.clone();
        tags.extend(vec![
            KeyValue::new(TELEMETRY_SDK_NAME, env!("CARGO_PKG_NAME").to_string()),
            KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
            KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        ]);

        if let Some(service_name) = self.service_name {
            // TODO: Is there a way to get the name of the bin crate using this?
            tags.push(KeyValue::new(SERVICE_NAME, service_name));
        }

        let trace_config = self
            .trace_config
            .unwrap_or_default()
            .with_resource(Resource::new(tags));

        let pipeline = opentelemetry_otlp::new_exporter()
            .http()
            .with_http_client(reqwest::Client::new())
            .with_endpoint(url)
            .with_headers(headers)
            .with_timeout(self.timeout.unwrap_or(Duration::from_secs(3)));
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(pipeline)
            .with_trace_config(trace_config)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        Ok(tracer)
    }
}

#[cfg(test)]
mod tests {

    use tracing_subscriber::Registry;

    use super::{Error, *};
    fn cache_axiom_env() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut saved_env = std::env::vars().collect::<HashMap<_, _>>();
        // Cache AXIOM env vars and remove from env for test
        for ref key in std::env::vars().map(|(key, _)| key) {
            if key.starts_with("AXIOM") {
                saved_env.insert(key.clone(), std::env::var(key)?);
                std::env::remove_var(key);
            }
        }

        Ok(saved_env)
    }

    fn restore_axiom_env(saved_env: HashMap<String, String>) {
        for (key, value) in saved_env {
            std::env::set_var(key, value);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_no_env_skips_env_variables() -> Result<(), Error> {
        let builder = Builder::default();
        assert_eq!(builder.token, None);
        assert_eq!(builder.dataset_name, None);
        assert_eq!(builder.url, None);

        let err: Result<Tracer, Error> = Builder::default().tracer();
        matches!(err, Err(Error::MissingToken));

        let builder = Builder::default()
            .with_token("xaat-snot")?
            .with_dataset("test")?;
        matches!(builder.tracer(), Err(Error::MissingDataset));

        let builder = Builder::default()
            .with_token("xaat-snot")?
            .with_dataset("test")?;
        matches!(builder.tracer(), Err(Error::MissingDataset));

        let builder = Builder::default()
            .with_token("xaat-snot")?
            .with_dataset("test")?;
        assert!(builder.tracer().is_ok());

        matches!(
            Builder::default().with_url("<invalid>"),
            Err(Error::InvalidUrl(url::ParseError::RelativeUrlWithoutBase))
        );

        Ok(())
    }

    #[tokio::test]
    async fn with_env_respects_env_variables() -> Result<(), Box<dyn std::error::Error>> {
        let cached_env = cache_axiom_env()?;

        let err = Builder::default().tracer();
        matches!(err, Err(Error::EnvVarMissing("AXIOM_TOKEN")));

        std::env::set_var("AXIOM_TOKEN", "xaat-snot");
        let err = Builder::default().tracer();
        matches!(err, Err(Error::EnvVarMissing("AXIOM_DATASET")));

        std::env::set_var("AXIOM_DATASET", "test");
        let ok = Builder::default().with_env()?.tracer();
        assert!(ok.is_ok());

        // NOTE We let this hang wet rather than fake the endpoint as
        // the tracer will try to connect to it and this may hang the
        // test if the endpoint is not reachable.
        //
        // std::env::set_var("AXIOM_URL", "http://localhost:8080");
        // let ok = Builder::new().tracer();
        // assert!(ok.is_ok());

        restore_axiom_env(cached_env);
        Ok(())
    }

    #[test]
    fn test_missing_token() {
        matches!(Builder::default().tracer(), Err(Error::MissingToken));
    }

    #[test]
    fn test_empty_token() {
        matches!(Builder::default().with_token(""), Err(Error::EmptyToken));
    }

    #[test]
    fn test_invalid_token() {
        matches!(
            Builder::default().with_token("invalid"),
            Err(Error::InvalidToken)
        );
    }

    #[test]
    fn test_invalid_url() -> Result<(), Error> {
        matches!(
            Builder::default()
                .with_token("xaat-123456789")?
                .with_dataset("test")?
                .with_url("<invalid>"),
            Err(Error::InvalidUrl(_))
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token() -> Result<(), Error> {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let result = Builder::default()
            .with_dataset("test")?
            .with_token("xaat-123456789")?
            .build::<Registry>();

        assert!(result.is_ok(), "{:?}", result.err());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token_env() -> Result<(), Error> {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let env_backup = env::var("AXIOM_TOKEN");
        env::set_var("AXIOM_TOKEN", "xaat-1234567890");

        let result = Builder::default()
            .with_dataset("test")?
            .with_env()?
            .build::<Registry>();

        if let Ok(token) = env_backup {
            env::set_var("AXIOM_TOKEN", token);
        }

        assert!(result.is_ok(), "{:?}", result.err());
        Ok(())
    }
}
