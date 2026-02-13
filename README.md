// README.md

# Rust Hello World Recipe App
Simple Rust API using Axum with a single endpoint that reads from and writes to a PostgreSQL database. Used within [Rust Hello World recipe](https://app.zerops.io/recipes/rust-hello-world) for [Zerops](https://zerops.io) platform.

## Integration Guide

<!-- #ZEROPS_EXTRACT_START:integration-guide# -->

### 1. Adding `zerops.yaml`
The main application configuration file you place at the root of your repository, it tells Zerops how to build, deploy and run your application.

```yaml
zerops:
  # Defining production setup, that will run the built application.
  - setup: prod
    build:
      # Using Rust build base image, that has Rust (and cargo) pre-installed.
      base: rust@latest
      buildCommands:
        # Build the application in release mode for optimal performance.
        - cargo build --release
      deployFiles:
        # We only need the compiled binary for production.
        - target/release/rust-hello-world-app
    run:
      # Using the same Rust base for the runtime environment.
      base: rust@latest
      ports:
        - port: 3000
          # Our app is an HTTP API. Mark the port as HTTP
          # so we can enable public HTTPS access.
          httpSupport: true
      # Execute the production binary directly from its build path.
      start: ./target/release/rust-hello-world-app

  # Dev setup is for remote development or AI agent use-cases.
  - setup: dev
    build:
      base: rust@latest
      buildCommands:
        # Build in debug mode for development.
        - cargo build
      deployFiles:
        # For development, we include the debug binary and the source code 
        # to allow for debugging and live changes inside the container.
        - target/debug/rust-hello-world-app
        - src
        - Cargo.toml
        - Cargo.lock
    run:
      base: rust@latest
      ports:
        - port: 3000
          httpSupport: true
      envVariables:
        # Enable full backtraces for easier debugging during development.
        RUST_BACKTRACE: "1"
      # Start the debug version of the application.
      start: ./target/debug/rust-hello-world-app
```
<!-- #ZEROPS_EXTRACT_END:integration-guide# -->