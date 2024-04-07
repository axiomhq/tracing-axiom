use crate::Error;
use opentelemetry::{Key, KeyValue};
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
    marker::PhantomData,
    time::Duration,
};
use tracing_core::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    layer::{Layered, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer, Registry,
};

const CLOUD_URL: &str = "https://api.axiom.co";

/// A layer that sends traces to Axiom via the `OpenTelemetry` protocol.
/// The layer cleans up the `OpenTelemetry` global tracer provider on drop.
type AxiomOpenTelemetryComposedLayer<S> =
    Layered<OpenTelemetryLayer<S, Tracer>, AxiomOpenTelemetryLayer<S>, S>;

/// A layer that sends traces to Axiom via the `OpenTelemetry` protocol.
/// The layer cleans up the `OpenTelemetry` global tracer provider on drop.
pub struct AxiomOpenTelemetryLayer<S>(PhantomData<S>);
impl<S> Default for AxiomOpenTelemetryLayer<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S> Layer<S> for AxiomOpenTelemetryLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
    Self: 'static,
{
}

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
    url: Option<String>,
    tags: Vec<KeyValue>,
    trace_config: Option<TraceConfig>,
    service_name: Option<String>,
    no_env: bool,
}

#[allow(clippy::match_same_arms)] // We want clarity here
fn resolve_configurable(
    should_check_environment: bool,
    env_var_name: &'static str,
    explicit_var: &Option<String>,
    predicate_check: fn(value: &Option<String>) -> Result<String, Error>,
) -> Result<String, Error> {
    match (
        should_check_environment,
        env::var(env_var_name),
        explicit_var,
    ) {
        // If we're skipping the environment variables, we need to have an explicit var
        (false, _, maybe_ok_var) => match predicate_check(maybe_ok_var) {
            Ok(valid_var) => Ok(valid_var),
            Err(err) => Err(err),
        },
        // If we respect the environment variables, and token is not set explicitly, use them
        (true, Ok(maybe_ok_var), _) => match predicate_check(&Some(maybe_ok_var)) {
            Ok(valid_var) => Ok(valid_var),
            Err(err) => Err(err),
        },
        // If env or programmatic token are invalid, fail and bail
        (true, Err(VarError::NotPresent), &None) => Err(Error::EnvVarMissing(env_var_name)),
        (true, Err(VarError::NotPresent), maybe_ok_var) => match predicate_check(maybe_ok_var) {
            Ok(valid_var) => Ok(valid_var),
            Err(err) => Err(err),
        },
        (true, Err(VarError::NotUnicode(_)), _) => {
            Err(Error::EnvVarNotUnicode(env_var_name.to_string()))
        }
    }
}

impl Builder {
    /// Create a new Builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            url: Some(CLOUD_URL.to_string()),
            ..Default::default()
        }
    }

    /// Set the Axiom dataset name to use. The dataset name is the name of the
    /// persistent dataset in Axiom cloud that will store the traces and make
    /// them available for querying using APL, the Axiom SDK or the Axiom CLI.
    #[must_use]
    pub fn with_dataset(mut self, dataset_name: impl Into<String>) -> Self {
        self.dataset_name = Some(dataset_name.into());
        self
    }

    /// Set the Axiom API token to use.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the Axiom API URL to use. Defaults to Axiom Cloud.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
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

    /// Don't fall back to environment variables.
    #[must_use]
    pub fn no_env(mut self) -> Self {
        self.no_env = true;
        self
    }

    /// Set the resource tags for the open telemetry tracer that publishes to Axiom.
    /// These tags will be added to all spans.
    ///
    /// # Errors
    ///
    /// Returns an error if a key is invalid.
    #[must_use]
    pub fn with_tags(mut self, tags: &[(&str, &str)]) -> Self {
        self.tags = tags
            .iter()
            .map(|(k, v)| KeyValue::new(Key::from((*k).to_string()), (*v).to_string()))
            .collect::<Vec<_>>();
        self
    }

    /// Initialize the global subscriber. This panics if the initialization was
    /// unsuccessful, likely because a global subscriber was already installed or
    /// `AXIOM_TOKEN` is not set or invalid.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization was unsuccessful, likely because
    /// a global subscriber was already installed or `AXIOM_TOKEN` is not set or
    /// invalid.
    ///
    pub fn init(self) -> Result<(), Error> {
        let layer = self.layer()?;
        Registry::default().with(layer).try_init()?;
        Ok(())
    }

    /// Create a layer which sends traces to Axiom and a Guard which will shut
    /// down the tracer provider on drop.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization was unsuccessful, likely because
    /// a global subscriber was already installed or `AXIOM_TOKEN` is not set or
    /// invalid.
    ///
    pub fn layer<S>(self) -> Result<AxiomOpenTelemetryComposedLayer<S>, Error>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        let tracer = self.tracer()?;
        let inner_layer: OpenTelemetryLayer<S, Tracer> =
            tracing_opentelemetry::layer().with_tracer(tracer);
        let layer = AxiomOpenTelemetryLayer::default().and_then(inner_layer);
        Ok(layer)
    }

    fn resolve_token(&self) -> Result<String, Error> {
        let token = &self.token;
        resolve_configurable(!self.no_env, "AXIOM_TOKEN", token, |token| match token {
            Some(token) if token.is_empty() => Err(Error::EmptyToken),
            Some(token) if !token.starts_with("xaat-") => Err(Error::InvalidToken),
            Some(token) => Ok(token.clone()),
            None => Err(Error::MissingToken),
        })
    }

    fn resolve_dataset_name(&self) -> Result<String, Error> {
        let dataset_name = &self.dataset_name;
        resolve_configurable(
            !self.no_env,
            "AXIOM_DATASET",
            dataset_name,
            |dataset_name| match dataset_name {
                Some(dataset_name) if dataset_name.is_empty() => Err(Error::EmptyDatasetName),
                Some(dataset_name) => Ok(dataset_name.clone()),
                None => Err(Error::MissingDatasetName),
            },
        )
    }

    fn resolve_axiom_url(&self) -> Result<String, Error> {
        let url = &self.url;
        resolve_configurable(!self.no_env, "AXIOM_URL", url, |url| match url {
            Some(url) => Ok(url.clone()),
            None => Ok(CLOUD_URL.to_string()),
        })
    }

    fn tracer(self) -> Result<Tracer, Error> {
        let token = self.resolve_token()?;
        let dataset_name = self.resolve_dataset_name()?;
        let url = self.resolve_axiom_url()?;

        let url = url.parse::<Url>()?;

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
            tags.push(KeyValue::new(SERVICE_NAME, service_name));
        }

        let trace_config = self
            .trace_config
            .unwrap_or_default()
            .with_resource(Resource::new(tags));

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
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        Ok(tracer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let builder = Builder::new().no_env();
        assert!(builder.no_env);
        assert_eq!(builder.token, None);
        assert_eq!(builder.dataset_name, None);
        assert_eq!(builder.url, Some("https://api.axiom.co".into()));

        let err: Result<Tracer, Error> = Builder::new().no_env().tracer();
        matches!(err, Err(Error::MissingToken));

        let mut builder = Builder::new().no_env();
        builder.token = Some("xaat-snot".into());
        let err = builder.tracer();
        matches!(err, Err(Error::MissingDatasetName));

        let mut builder = Builder::new().no_env();
        builder.token = Some("xaat-snot".into());
        builder.dataset_name = Some("test".into());
        let ok = builder.tracer();
        assert!(ok.is_ok());

        let mut builder = Builder::new().no_env();
        builder.token = Some("xaat-snot".into());
        builder.dataset_name = Some("test".into());
        builder.url = Some("<invalid>".into());
        let err = builder.tracer();
        assert!(err.is_err());
        matches!(
            err,
            Err(Error::InvalidUrl(url::ParseError::RelativeUrlWithoutBase))
        );

        Ok(())
    }

    #[tokio::test]
    async fn with_env_respects_env_variables() -> Result<(), Box<dyn std::error::Error>> {
        let cached_env = cache_axiom_env()?;

        let builder = Builder::new();
        assert!(!builder.no_env);

        let err = Builder::new().tracer();
        matches!(err, Err(Error::EnvVarMissing("AXIOM_TOKEN")));

        std::env::set_var("AXIOM_TOKEN", "xaat-snot");
        let err = Builder::new().tracer();
        matches!(err, Err(Error::EnvVarMissing("AXIOM_DATASET")));

        std::env::set_var("AXIOM_DATASET", "test");
        let ok = Builder::new().tracer();
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
        matches!(Builder::new().no_env().init(), Err(Error::MissingToken));
    }

    #[test]
    fn test_empty_token() {
        matches!(
            Builder::new().no_env().with_token("").init(),
            Err(Error::EmptyToken)
        );
    }

    #[test]
    fn test_invalid_token() {
        matches!(
            Builder::new().no_env().with_token("invalid").init(),
            Err(Error::InvalidToken)
        );
    }

    #[test]
    fn test_invalid_url() {
        matches!(
            Builder::new()
                .no_env()
                .with_token("xaat-123456789")
                .with_dataset("test")
                .with_url("<invalid>")
                .init(),
            Err(Error::InvalidUrl(_))
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token() {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let result = Builder::new()
            .no_env()
            .with_dataset("test")
            .with_token("xaat-123456789")
            .layer::<Registry>();

        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_valid_token_env() {
        // Note that we can't test the init/try_init funcs here because OTEL
        // gets confused with the global subscriber.

        let env_backup = env::var("AXIOM_TOKEN");
        env::set_var("AXIOM_TOKEN", "xaat-1234567890");

        let result = Builder::new().with_dataset("test").layer::<Registry>();

        if let Ok(token) = env_backup {
            env::set_var("AXIOM_TOKEN", token);
        }

        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    #[cfg(feature = "unstable")]
    fn test_env_var() {
        use std::ffi::OsStr;
        let result = resolve_configurable(true, "BAD_ENV_VAR", &None, |_| Ok("ok".to_string()));
        assert_eq!(Err(Error::EnvVarMissing("BAD_ENV_VAR".to_string())), result);
        // NOTE unstable feature - so we cannot assert this on stable yet
        let non_unicode_utf8_str =
            unsafe { OsStr::from_encoded_bytes_unchecked(b"\xFF\xFE\x41\x42snot") }; // No NUL bytes!
        env::set_var("BAD_ENV_VAR", non_unicode_utf8_str);
        let result = resolve_configurable(true, "BAD_ENV_VAR", &Some("ok".to_string()), |_| {
            Ok("ok".to_string())
        });
        assert_eq!(
            Err(Error::EnvVarNotUnicode("BAD_ENV_VAR".to_string())),
            result
        );
    }
}
