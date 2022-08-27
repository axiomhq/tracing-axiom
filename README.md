# tracing-axiom

[![CI](https://github.com/axiomhq/tracing-axiom/workflows/CI/badge.svg)](https://github.com/axiomhq/tracing-axiom/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/tracing-axiom.svg)](https://crates.io/crates/tracing-axiom)
[![docs.rs](https://docs.rs/tracing-axiom/badge.svg)](https://docs.rs/tracing-axiom/)
[![License](https://img.shields.io/crates/l/tracing-axiom)](LICENSE-APACHE)

The tracing layer for shipping traces to Axiom.

## Install

Add the following to your `Cargo.toml`:

```toml
[dependencies]
tracing-axiom = "0.2"
```

## Quickstart

Expose an API token with ingest permission under `AXIOM_TOKEN` and initialize
the exporter like this:

```rust
#[tokio::main]
async fn main() {
    let _guard = tracing_axiom::init(); // or try_init() to handle errors
    say_hello();
}

#[tracing::instrument]
pub fn say_hello() {
    tracing::info!("Hello, world!");
}
```

> **Note**: Due to a limitation of an underlying library, [events outside of a 
> span are not recorded](https://docs.rs/tracing-opentelemetry/0.17.4/src/tracing_opentelemetry/layer.rs.html#807).

## Kitchen Sink Full Configuration

Here's a full configuration:

```rust
use opentelemetry::sdk::trace;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let trace_config = trace::Config::default()
    .with_max_events_per_span(42);

  let _guard = tracing_axiom::builder()
    .with_token("xaat-123456789")
    .with_url("https://my-axiom.example.org")
    .with_service_name("my-service")
    .with_trace_config(trace_config)
    .try_init()?;
  Ok(())
}
```

If you want to use other layers next to Axiom in your tracing configuration, 
check out the [fmt example](./examples/fmt).

## Under The Hood

This library uses [OpenTelemetry](https://opentelemetry.io) to send data to
Axiom.
You can set this up yourself if you want to, but make sure to use the OTLP 
format with the http transport and set the endpoint to
`https://cloud.axiom.co/api/v1/traces`.
A good entrypoint is the
[`opentelemetry-otlp`](https://docs.rs/opentelemetry-otlp) crate.

## Features

The following are a list of
[Cargo features](https://doc.rust-lang.org/stable/cargo/reference/features.html#the-features-section)
that can be enabled or disabled:

- **default-tls** _(enabled by default)_: Provides TLS support to connect
  over HTTPS.
- **native-tls**: Enables TLS functionality provided by `native-tls`.
- **rustls-tls**: Enables TLS functionality provided by `rustls`.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.