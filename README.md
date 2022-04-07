# transactions

A simple toy payments engine that reads a series of transactions from a CSV,
updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV.

## About

A fun challenge of implementing a payment engine.

### Example

```sh
cargo run -- transactions.csv
```

## Feature Flags

### Default Features

* **[clap](https://github.com/clap-rs/clap)**: Command line argument parsing
   * This feature will guide users how to use the application.
   * When not enabled:
      * A single file argument is required.
      * Improper arguments will panic.
   * `cargo run -- --help`

#### Optional features

* **[env_logger](https://github.com/env-logger-rs/env_logger/)**: for debug output.
   * `RUST_LOG=trace cargo run --features env_logger -- tractions.csv`

## Tests

There are CLI and and integration tests available to perform some limited "good
path" testing.

* The CLI tests exercise the executable and parse for expected output.
* The integration tests run the functions provided by the library portion of the code.
