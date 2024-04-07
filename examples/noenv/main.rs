use tracing::{info, instrument};
use uuid::Uuid;

#[instrument]
fn say_hi(id: Uuid, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_axiom::builder()
        .with_service_name("noenv")
        .with_tags(&[("aws_region", "us-east-1")]) // Set otel tags
        .with_dataset("tracing-axiom-examples") // Set dataset
        .with_token("xaat-some-valid-token") // Set API token
        .with_url("http://localhost:4318") // Set URL, can be changed to any OTEL endpoint
        .init()?; // Initialize tracing

    let uuid = Uuid::new_v4();
    say_hi(uuid, "world");

    // do something with result ...

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
