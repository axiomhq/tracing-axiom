use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::{info, instrument};
use tracing_subscriber::Registry;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[instrument]
fn say_hi(id: u64, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

fn setup_tracing(tags: &[(&'static str, &'static str)]) -> Result<(), tracing_axiom::Error> {
    info!("Axiom OpenTelemetry tracing endpoint is configured:");
    // Setup an AWS CloudWatch compatible tracing layer
    let cloudwatch_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .without_time()
        .with_target(false);

    // Setup an Axiom OpenTelemetry compatible tracing layer
    let tag_iter = tags.iter().copied();
    let axiom_layer = tracing_axiom::builder("layers")
        .with_tags(tag_iter)
        .build()?;

    // Setup our multi-layered tracing subscriber
    Registry::default()
        .with(axiom_layer)
        .with(cloudwatch_layer)
        .init();

    global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(())
}

const TAGS: &[(&str, &str)] = &[
    ("aws_region", "us-east-1"), // NOTE - example for illustrative purposes only
];

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing(TAGS)?; // NOTE we depend on environment variable

    say_hi(42, "world");

    // do something with result ...

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
