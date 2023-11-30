use tracing::{info, instrument};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_axiom::init()?;
    say_hello();

    Ok(())
}

#[instrument]
fn say_hello() {
    info!("hello world")
}
