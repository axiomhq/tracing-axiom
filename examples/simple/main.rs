use tracing::{error, instrument};

#[instrument]
fn say_hello() {
    error!("hello world")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_axiom::init()?;
    say_hello();

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
