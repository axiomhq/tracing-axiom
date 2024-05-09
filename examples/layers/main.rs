use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing::{info, instrument};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;
use uuid::Uuid;

#[instrument]
fn say_hi(id: Uuid, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

fn setup_tracing(otel_is_configured: bool, tags: &[(&str, &str)]) -> Result<(), anyhow::Error> {
    if otel_is_configured {
        info!("Axiom OpenTelemetry tracing endpoint is configured:");
        // Setup an AWS CloudWatch compatible tracing layer
        let cloudwatch_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_ansi(false)
            .without_time()
            .with_target(false);

        // Setup an Axiom OpenTelemetry compatible tracing layer
        let axiom_layer = tracing_axiom::builder()
            .with_service_name("layers")
            .with_tags(tags)
            .layer()?;

        // Setup our multi-layered tracing subscriber
        Registry::default()
            .with(axiom_layer)
            .with(cloudwatch_layer)
            .init();
    } else {
        info!("OpenTelemetry is not configured: Using AWS CloudWatch savvy format",);
        tracing_subscriber::fmt()
            .json()
            .with_max_level(tracing::Level::INFO)
            .with_current_span(false)
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .init();
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(())
}

const TAGS: &[(&str, &str)] = &[
    ("aws_region", "us-east-1"), // NOTE - example for illustrative purposes only
];

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing(true, TAGS)?; // NOTE we depend on environment variable

    let uuid = Uuid::new_v4();
    say_hi(uuid, "world");

    // do something with result ...

    // Ensure that the tracing provider is shutdown correctly
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
