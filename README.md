# Rust Hello World Recipe App

<!-- #ZEROPS_EXTRACT_START:intro# -->
Simple Actix Web API connected to PostgreSQL, with health check verifying database connectivity and querying migrated data.
Used within [Rust Hello World recipe](https://app.zerops.io/recipes/rust-hello-world) for [Zerops](https://zerops.io) platform.
<!-- #ZEROPS_EXTRACT_END:intro# -->

⬇️ **Full recipe page and deploy with one-click**

[![Deploy on Zerops](https://github.com/zeropsio/recipe-shared-assets/blob/main/deploy-button/light/deploy-button.svg)](https://app.zerops.io/recipes/rust-hello-world?environment=small-production)

![rust cover](https://github.com/zeropsio/recipe-shared-assets/blob/main/covers/svg/cover-rust.svg)

## Integration Guide

<!-- #ZEROPS_EXTRACT_START:integration-guide# -->

### 1. Adding `zerops.yaml`
The main application configuration file you place at the root of your repository, it tells Zerops how to build, deploy and run your application.

```yaml
zerops:
  # Production setup — compile optimized binaries, deploy minimal footprint.
  # Matching build/runtime Rust stable prevents native linking mismatches.
  - setup: prod
    build:
      base: rust@stable

      # Redirect cargo registry into project tree so Zerops can cache it.
      # build.envVariables persists across all build phases — unlike
      # inline prefixes or prepareCommands exports.
      envVariables:
        CARGO_HOME: ./.cargo

      buildCommands:
        # --release for optimized binary, --locked validates Cargo.lock
        # against Cargo.toml — prevents unexpected dep updates in prod.
        # Builds both 'rust-hello-world' and 'migrate' binaries.
        - cargo build --release --locked

      deployFiles:
        - ./target/release/rust-hello-world
        - ./target/release/migrate

      cache:
        # Paths match CARGO_HOME above. Caching both registry (downloaded
        # sources) and target (compiled deps) speeds up subsequent builds.
        - .cargo/registry
        - target

    # Readiness check: container must serve GET / before the project
    # balancer routes traffic to it — prevents serving during startup.
    deploy:
      readinessCheck:
        httpGet:
          port: 8080
          path: /

    run:
      base: rust@stable

      # Migrations run once per deploy version across all containers.
      # In initCommands (not buildCommands) so migration and code deploy
      # atomically — a failed deploy won't leave a migrated DB with old code.
      initCommands:
        - zsc execOnce ${appVersionId} -- ./target/release/migrate

      ports:
        - port: 8080
          httpSupport: true

      envVariables:
        DB_NAME: db
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}

      start: ./target/release/rust-hello-world

  # Development setup — deploy full source for interactive work via SSH.
  # Developer runs 'cargo run' themselves; Zerops prepares the workspace.
  - setup: dev
    build:
      base: rust@stable
      os: ubuntu

      envVariables:
        CARGO_HOME: ./.cargo

      buildCommands:
        # Only fetch dependencies — no compilation.
        # Developer compiles on demand via SSH.
        - cargo fetch

      deployFiles:
        - ./

      cache:
        - .cargo/registry

    run:
      # rust@stable on Ubuntu — developer needs cargo/rustc via SSH,
      # and Ubuntu provides a richer toolset for interactive development.
      base: rust@stable
      os: ubuntu

      # Same execOnce migration pattern as prod. Compiles migration binary
      # on first deploy (one-time cost), then runs it against the database.
      initCommands:
        - zsc execOnce ${appVersionId} -- cargo run --bin migrate

      ports:
        - port: 8080
          httpSupport: true

      envVariables:
        CARGO_HOME: /var/www/.cargo
        DB_NAME: db
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}

      # No app started — developer connects via SSH and runs 'cargo run'
      start: zsc noop --silent
```
<!-- #ZEROPS_EXTRACT_END:integration-guide# -->


<!-- #ZEROPS_EXTRACT_START:knowledge-base# -->
### Gotchas

- **Rust build container has no `pkg-config` / `libssl-dev`** — any crate
  that defaults to `native-tls` (most prominently `reqwest`, but also
  `hyper-tls`, `async-native-tls`, etc.) tries to link system OpenSSL
  via `pkg-config` and fails the build with `Could not find directory of
  OpenSSL installation` and `pkg-config command could not be found`.
  Switch to `rustls-tls` (pure Rust TLS, no system deps):

  ```toml
  reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
  ```

  Same pattern for any HTTP client crate that defaults to native TLS:
  disable default features, then enable `rustls-tls` (or the crate's
  equivalent rustls feature) plus only the JSON / HTTP-method / etc.
  features actually needed. The Zerops Rust runtime is intentionally
  minimal — there is no apt-install-libssl-dev escape hatch in the
  build phase.
<!-- #ZEROPS_EXTRACT_END:knowledge-base# -->