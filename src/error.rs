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
    EnvVarMissing(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Coerce partial_eq as inner error is not comparable
            (Error::TraceError(_), Error::TraceError(_))
            | (Error::InitErr(_), Error::InitErr(_))
            // Our errors are comparable, natch
            | (Error::MissingToken, Error::MissingToken)
            | (Error::EmptyToken, Error::EmptyToken)
            | (Error::InvalidToken, Error::InvalidToken)
            | (Error::MissingDatasetName, Error::MissingDatasetName)
            | (Error::EmptyDatasetName, Error::EmptyDatasetName) => true,
            (Error::InvalidUrl(lhs), Error::InvalidUrl(rhs)) => lhs == rhs,
            (Error::EnvVarNotUnicode(lhs), Error::EnvVarNotUnicode(rhs))
            | (Error::EnvVarMissing(lhs), Error::EnvVarMissing(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}
