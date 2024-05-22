use tracing::{info, instrument};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let axiom_layer = tracing_axiom::builder("fmt").build()?;
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(axiom_layer)
        .try_init()?;

    say_hello();

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

#[instrument]
fn say_hello() {
    info!("hello world")
}
