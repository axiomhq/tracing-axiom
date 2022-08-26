use tracing::{info, instrument};

#[tokio::main]
async fn main() {
    let _guard = tracing_axiom::init();
    say_hello();
}

#[instrument]
fn say_hello() {
    info!("hello world")
}
