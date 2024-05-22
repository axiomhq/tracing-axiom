use tracing::{error, instrument};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, Registry};

#[instrument]
fn say_hello() {
    error!("hello world")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let axiom_layer = tracing_axiom::default("simple")?;

    Registry::default().with(axiom_layer).init();

    say_hello();

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
