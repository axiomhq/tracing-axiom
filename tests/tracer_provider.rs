use axiom_rs::Client;
use tracing::{info, instrument};
use uuid::Uuid;

#[instrument]
fn say_hi(id: Uuid, name: impl Into<String> + std::fmt::Debug) {
    info!(?id, "Hello, {}!", name.into());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_traces_ingest() -> Result<(), Box<dyn std::error::Error>> {
    let guard = tracing_axiom::try_init()?;
    let uuid = Uuid::new_v4();
    say_hi(uuid, "world");
    drop(guard); // flush traces and shutdown the tracer provider.

    let client = Client::new()?;
    let res = client
        .datasets
        .apl_query(
            format!(
                r#"['_traces'] | where name == "say_hi" and ['attributes.id'] == "{}""#,
                uuid.to_string()
            ),
            None,
        )
        .await?;
    assert_eq!(res.matches.len(), 1);
    Ok(())
}
