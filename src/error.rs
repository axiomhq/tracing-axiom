use opentelemetry::trace;
use tracing_subscriber::util::TryInitError;

/// The error type for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to configure the tracer.
    #[error("Failed to configure tracer: {0}")]
    TraceError(#[from] trace::TraceError),

    /// Failed to initialize the tracing-subscriber registry.
    #[error("Failed to initialize registry: {0}")]
    InitErr(#[from] TryInitError),

    /// The required Axiom API token is missing.
    #[error("Token is missing")]
    MissingToken,

    /// The required Axiom API token is empty.
    #[error("Token is empty")]
    EmptyToken,

    /// The required Axiom token is missing.
    #[error("Invalid token (please provide a valid API token)")]
    InvalidToken,

    /// The required Axiom dataset name is missing.
    #[error("Dataset name is missing")]
    MissingDatasetName,

    /// The required Axiom dataset name is empty.
    #[error("Dataset name is empty")]
    EmptyDatasetName,

    /// The required Axiom dataset name is invalid.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// The environment variable is malformed unicode.
    #[error("Environment variable {0} contains invalid non Unciode ( UTF-8 ) content")]
    EnvVarNotUnicode(String),

    /// The environment variable is not present.
    #[error("Environment variable {0} is required but missing")]
    EnvVarMissing(&'static str),
}
