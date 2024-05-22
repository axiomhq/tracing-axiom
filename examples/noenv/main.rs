use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, Registry};

#[instrument]
fn say_hi(id: u64, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let axiom_layer = tracing_axiom::builder("noenv")
        .with_tags([("aws_region", "us-east-1")].iter().copied()) // Set otel tags
        .with_dataset("tracing-axiom-examples")? // Set dataset
        .with_token("xaat-some-valid-token")? // Set API token
        .with_url("http://localhost:4318")? // Set URL, can be changed to any OTEL endpoint
        .build()?; // Initialize tracing

    Registry::default().with(axiom_layer).init();

    say_hi(42, "world");

    // do something with result ...

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
