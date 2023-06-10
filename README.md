# :warning: EXPERIMENTAL :warning:

This is still experimental. Traces are ingested raw into Axiom and there's no nice way to view them (yet). 

----

![tracing-axiom: The official Rust tracing layer for Axiom](.github/images/banner-dark.svg#gh-dark-mode-only)
![tracing-axiom: The official Rust tracing layer for Axiom](.github/images/banner-light.svg#gh-light-mode-only)

<div align="center">

[![docs.rs](https://docs.rs/tracing-axiom/badge.svg)](https://docs.rs/tracing-axiom/)
[![build](https://img.shields.io/github/workflow/status/axiomhq/tracing-axiom/CI?ghcache=unused)](https://github.com/axiomhq/tracing-axiom/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/tracing-axiom.svg)](https://crates.io/crates/tracing-axiom)
[![License](https://img.shields.io/crates/l/tracing-axiom)](LICENSE-APACHE)

</div>

[Axiom](https://axiom.co) unlocks observability at any scale.

- **Ingest with ease, store without limits:** Axiom’s next-generation datastore enables ingesting petabytes of data with ultimate efficiency. Ship logs from Kubernetes, AWS, Azure, Google Cloud, DigitalOcean, Nomad, and others.
- **Query everything, all the time:** Whether DevOps, SecOps, or EverythingOps, query all your data no matter its age. No provisioning, no moving data from cold/archive to “hot”, and no worrying about slow queries. All your data, all. the. time.
- **Powerful dashboards, for continuous observability:** Build dashboards to collect related queries and present information that’s quick and easy to digest for you and your team. Dashboards can be kept private or shared with others, and are the perfect way to bring together data from different sources

For more information check out the [official documentation](https://axiom.co/docs).

## Usage

Add the following to your Cargo.toml:

```toml
[dependencies]
tracing-axiom = "0.3"
```

Create a dataset in Axiom and export the name as `AXIOM_DATASET`.
Then create an API token with ingest permission into that dataset in
[the Axiom settings](https://cloud.axiom.co/settings/profile) and export it as
`AXIOM_TOKEN`.

Now you can set up tracing in one line like this:

```rust
#[tokio::main]
async fn main() {
    let _guard = tracing_axiom::init(); // or try_init() if you want to handle errors
    say_hello();
}

#[tracing::instrument]
pub fn say_hello() {
    tracing::info!("Hello, world!");
}
```

For further examples, head over to the [examples](examples) directory.

> **Note**: Due to a limitation of an underlying library, [events outside of a 
> span are not recorded](https://docs.rs/tracing-opentelemetry/0.17.4/src/tracing_opentelemetry/layer.rs.html#807).

## Features

The following are a list of
[Cargo features](https://doc.rust-lang.org/stable/cargo/reference/features.html#the-features-section)
that can be enabled or disabled:

- **default-tls** _(enabled by default)_: Provides TLS support to connect
  over HTTPS.
- **native-tls**: Enables TLS functionality provided by `native-tls`.
- **rustls-tls**: Enables TLS functionality provided by `rustls`.

## FAQ & Troubleshooting
### How do I log traces to the console in addition to Axiom?
You can use this library to get a [`tracing-subscriber::layer`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html) 
and combine it with other layers, for example one that prints traces to the 
console.
You can see how this works in the [fmt example](./examples/fmt).

### Logs are not appearing in Axiom
[`init`](https://docs.rs/tracing-axiom/latest/tracing_axiom/fn.init.html), [`try_init`](https://docs.rs/tracing-axiom/latest/tracing_axiom/fn.try_init.html) and [`layer`](https://docs.rs/tracing-axiom/latest/tracing_axiom/struct.Builder.html#method.layer) all return a [`Guard`](https://docs.rs/tracing-axiom/latest/tracing_axiom/struct.Guard.html), which will shutdown the 
tracer provider on drop.
Logs won't be sent to Axiom if the `Guard` is dropped prematurely.
If you have a function that sets up observability, return the `Guard` up to the
main func to prevent it from being dropped.

### My test function hangs indefinitely
This can happen when you use `#[tokio::test]` as that defaults to a 
single-threaded executor, but the 
[`opentelemetry`](https://docs.rs/opentelemetry) crate requires a multi-thread
executor.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
