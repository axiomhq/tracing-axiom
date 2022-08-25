use tracing::{info, instrument};

#[tokio::main]
async fn main() {
    tracing_axiom::init();
    say_hello();
    tracing_axiom::shutdown();
}

#[instrument]
fn say_hello() {
    info!("hello world")
}
