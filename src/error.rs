use opentelemetry::trace;
use tracing_subscriber::util::TryInitError;

/// The error type for this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to configure tracer: {0}")]
    TraceError(#[from] trace::TraceError),
    #[error("Failed to initialize registry: {0}")]
    InitErr(#[from] TryInitError),
    #[error("Token is missing")]
    MissingToken,
    #[error("Token is empty")]
    EmptyToken,
    #[error("Invalid token (please provide a personal token)")]
    InvalidToken,
    #[error("Dataset name is missing")]
    MissingDatasetName,
    #[error("Dataset name is empty")]
    EmptyDatasetName,
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}
