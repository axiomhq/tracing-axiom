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
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tracing_axiom::init()?; // Set AXIOM_DATASET and AXIOM_TOKEN in your env!
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

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

/// Initialize a global subscriber which sends traces to Axiom.
///
/// It uses the environment variables `AXIOM_TOKEN` and optionally `AXIOM_URL`
/// to configure the endpoint.
/// If you want to manually set these, see [`Builder`].
///
/// # Errors
///
/// Errors if the initialization was unsuccessful, likely because a global
/// subscriber was already installed or `AXIOM_TOKEN` and/or `AXIOM_DATASET`
/// is not set or invalid.
pub fn init() -> Result<(), Error> {
    builder().try_init()
}

/// Create a new [`Builder`].
#[must_use]
pub fn builder() -> Builder {
    Builder::new()
}
