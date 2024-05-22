#![deny(warnings)]
#![deny(missing_docs)]
#![recursion_limit = "1024"]
#![deny(
    clippy::all,
    clippy::unwrap_used,
    clippy::unnecessary_unwrap,
    clippy::pedantic,
    clippy::mod_module_files
)]

//! Send traces to Axiom with a single line.
//!
//! # Example
//!
//! In a project that uses [Tokio](https://tokio.rs) and
//! [tracing](https://docs.rs/tracing) run `cargo add tracing-axiom` and
//! configure it like this:
//!
//! ```rust,no_run
//! use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, Registry};
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let axiom_layer = tracing_axiom::default("doctests")?; // Set AXIOM_DATASET and AXIOM_TOKEN in your env!
//!     Registry::default().with(axiom_layer).init();
//!     say_hello();
//!     Ok(())
//! }
//!
//! #[tracing::instrument]
//! pub fn say_hello() {
//!     tracing::info!("Hello, world!");
//! }
//! ```
//!
//! The example above gets the Axiom API token from the `AXIOM_TOKEN` env and
//! the dataset name from `AXIOM_DATASET`. For more advanced configuration, see [`builder()`].

mod builder;
mod error;

pub use builder::Builder;
pub use error::Error;
use opentelemetry_sdk::trace::Tracer;
use tracing_core::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

/// Creates a default [`OpenTelemetryLayer`] with a [`Tracer`] that sends traces to Axiom.
///
/// It uses the environment variables `AXIOM_TOKEN` and optionally `AXIOM_URL` and `AXIOM_DATASET`
/// to configure the endpoint.
/// If you want to manually set these or other attributres, use `builder()` or `builder_with_env()`.
///
/// # Errors
///
/// Errors if the initialization was unsuccessful, likely because a global
/// subscriber was already installed or `AXIOM_TOKEN` and/or `AXIOM_DATASET`
/// is not set or invalid.
pub fn default<S>(service_name: &str) -> Result<OpenTelemetryLayer<S, Tracer>, Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    builder_with_env(service_name)?.build()
}

/// Create a new [`Builder`] and set the configuratuin from the environment.
///
/// # Errors
/// If any of the environment variables are invalid, missing variables are not causing errors as
/// they can be set later.
pub fn builder_with_env(service_name: &str) -> Result<Builder, Error> {
    Ok(Builder::default()
        .with_env()?
        .with_service_name(service_name))
}

/// Create a new [`Builder`] with no defaults set.
#[must_use]
pub fn builder(service_name: &str) -> Builder {
    Builder::default().with_service_name(service_name)
}
