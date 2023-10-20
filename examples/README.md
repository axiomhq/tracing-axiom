# **Examples**

* [simple](./simple) - Uses defaults provided by Axiom and is a one line setup.
* [fmt](./fmt) - Uses layers with out of the box local formatting and Axiom remote endpoint.
* [sdk](./sdk) - Shows how tracing and the Axiom SDK can be used together.
* [layers](./layers) - The kitchen sink. If you have a rich tracing setup, just plug tracing-axiom into your existing setup.
* [noenv]('./noenv) - Example that does not use environment variables for tracing setup.

## Setup

The `sdk` and `layers` examples assume the existance of a dataset called `tracing-axiom-examples` in your axiom
environment. You can setup a dataset using the `Axiom Cli` ([docs](https://axiom.co/docs/reference/cli), [github](https://github.com/axiomhq/cli)) as follows:

```shell
axiom dataset create --name tracing-axiom-examples --description "Dataset for testing tracing axiom"
export AXIOM_DATASET=tracing-axiom-examples
```

If the environment variable is not set, the examples will fail with an error.
