//! Send traces to Axiom with a single line.
//!
//! # Example
//!
//! In a project that uses [Tokio](https://tokio.rs) and
//! [tracing](https://docs.rs/tracing) run `cargo add tracing-axiom` and
//! configure it like this:
//!
//! ```
//! #[tokio::main]
//! async fn main() {
//!     let _guard = tracing_axiom::init();
//!     say_hello();
//! }
//!
//! #[tracing::instrument]
//! pub fn say_hello() {
//!     tracing::info!("Hello, world!");
//! }
//! ```
//!
//! The example above gets the Axiom API token from the `AXIOM_TOKEN` env and
//! panics if setup fails. If you want to handle the error, use [`try_init`].
//! For more advanced configuration, see [`builder()`].

use lazy_static::lazy_static;
use url::Url;

mod builder;
mod error;

pub use builder::{Builder, Guard};
pub use error::Error;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

lazy_static! {
    pub(crate) static ref CLOUD_URL: Url = Url::parse("https://cloud.axiom.co").unwrap();
}

/// Initialize a global subscriber which sends traces to Axiom.
///
/// It uses the environment variables `AXIOM_TOKEN` and optionally `AXIOM_URL`
/// to configure the endpoint.
/// If you want to manually set these, see [`Builder`].
///
/// # Panics
///
/// Panics if the initialization was unsuccessful, likely because a global
/// subscriber was already installed or `AXIOM_TOKEN` is not set or invalid.
/// If you want to handle the error instead, use [`try_init`].
pub fn init() -> Guard {
    Builder::new().init()
}

/// Initialize a global subscriber which sends traces to Axiom.
///
/// It uses the environment variables `AXIOM_TOKEN` and optionally `AXIOM_URL`
/// to configure the endpoint.
/// If you want to manually set these, see [`Builder`].
///
/// # Errors
///
/// Returns an error if the initialization was unsuccessful, likely because a
/// global subscriber was already installed or `AXIOM_TOKEN` is not set or
/// invalid.
pub fn try_init() -> Result<Guard, Error> {
    Builder::new().try_init()
}

/// Create a new [`Builder`].
pub fn builder() -> Builder {
    Builder::new()
}
