use axiom_rs::Client;
use tracing::{info, instrument};
use uuid::Uuid;

#[instrument]
fn say_hi(id: Uuid, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_axiom::init()?;

    let dataset: String = std::env::var("AXIOM_DATASET")?;

    let uuid = Uuid::new_v4();
    say_hi(uuid, "world");

    let query = format!(r#"['{}'] | where name == "say_hi""#, dataset);

    let client = Client::new()?;
    let _result = client.datasets.apl_query(query, None).await?;

    // do something with result ...

    Ok(())
}
