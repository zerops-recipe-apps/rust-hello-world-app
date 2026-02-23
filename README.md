<!-- #ZEROPS_EXTRACT_START:intro# -->
# Rust Hello World Recipe App

Actix-Web application connected to PostgreSQL, with a health check endpoint at `/` that verifies database connectivity and queries data seeded by an idempotent migration. Used within [Rust Hello World recipe](https://app.zerops.io/recipes/rust-hello-world) for [Zerops](https://zerops.io) platform.

**Full recipe page and deploy with one-click**

[![Deploy on Zerops](https://github.com/zeropsio/recipe-shared-assets/blob/main/deploy-button/light/deploy-button.svg)](https://app.zerops.io/recipes/rust-hello-world?environment=small-production)

![Rust cover](https://github.com/zeropsio/recipe-shared-assets/blob/main/covers/svg/cover-rust.svg)
<!-- #ZEROPS_EXTRACT_END:intro# -->

<!-- #ZEROPS_EXTRACT_START:integration-guide# -->
## Integration Guide

### 1. Adding `zerops.yaml`
The main application configuration file you place at the root of your repository, it tells Zerops how to build, deploy and run your application.

```yaml
zerops:
  # Production setup — compile optimized release binaries and deploy minimal footprint.
  # Two binaries are built: the web server and the migration runner. Both land in
  # target/release/ and are deployed explicitly so the runtime container stays lean.
  # Contrast with the 'dev' setup below, which deploys full source for SSH development.
  - setup: prod
    build:
      base: rust@stable
      # Ubuntu provides glibc, which the Rust toolchain links against by default.
      # Alpine uses musl — running a glibc-linked binary there would fail with
      # "not found" or "Exec format error". Matching build and runtime OS avoids this.
      os: ubuntu

      # Redirect Cargo's registry into the project tree so Zerops can cache it
      # between builds. The default ~/.cargo is outside the project and unreachable
      # by the cache system. Must match the 'cache' path below.
      envVariables:
        CARGO_HOME: ./.cargo

      buildCommands:
        # --release: enables full optimizations (LLVM, inlining, dead-code elimination).
        # --locked: requires Cargo.lock to be committed and match Cargo.toml exactly —
        # prevents silent dependency drift between CI and production builds.
        # Both binaries defined in [[bin]] sections of Cargo.toml are built in one pass.
        - cargo build --release --locked

      deployFiles:
        # Deploy both release binaries explicitly. The runtime container needs the
        # web server to serve requests and the migration binary to run initCommands.
        # Using full paths (not the ~/strip trick) keeps the runtime layout explicit.
        - ./target/release/rust-hello-world
        - ./target/release/migrate

      cache:
        # Cache the downloaded crate registry. First build: ~2 min downloading crates.
        # Subsequent builds with cache: seconds. Path matches CARGO_HOME above.
        - .cargo/registry

    # readinessCheck verifies each new runtime container is healthy BEFORE the project
    # balancer routes traffic to it. Without this, users would see 502 errors during
    # the brief window between container start and application ready-to-serve.
    deploy:
      readinessCheck:
        httpGet:
          port: 3000
          path: /

    run:
      base: rust@stable
      # Ubuntu runtime matches the build OS — glibc-linked binaries require glibc.
      # The Rust toolchain in this image is unused at runtime (binary is pre-compiled),
      # but the ubuntu base is necessary for the glibc dependency.
      os: ubuntu

      # initCommands run on every container start, BEFORE the application starts.
      # Running migrations here (not in buildCommands) ensures the schema update and
      # the new binary are always deployed together — if deploy fails, both roll back.
      # zsc execOnce ensures only one container runs the migration when scaling to
      # multiple replicas, preventing race conditions on INSERT/CREATE TABLE.
      initCommands:
        - zsc execOnce ${ZEROPS_appVersionId} -- ./target/release/migrate

      ports:
        - port: 3000
          httpSupport: true

      # Zerops injects these as environment variables into each runtime container.
      # Variable names follow the {hostname}_{key} pattern — 'db' is the hostname
      # defined in the import.yaml, so db_hostname, db_port, etc.
      envVariables:
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}
        DB_NAME: db

      # Reserve ~40% of minRam as free headroom. Rust has no GC, but actix-web
      # worker threads allocate per-request. Without headroom, traffic spikes OOM-kill.
      verticalAutoscaling:
        minRam: 0.25
        minFreeRamGB: 0.1

      start: ./target/release/rust-hello-world

  # Development setup — deploy full source code so developers can SSH in and work
  # interactively. The build container only fetches dependencies; no compilation happens
  # here because the developer will compile and run the app themselves via SSH.
  - setup: dev
    build:
      base: rust@stable
      os: ubuntu

      envVariables:
        # Same CARGO_HOME as prod — keeps the cache path consistent across setups.
        # At runtime this is overridden to an absolute path (see run.envVariables).
        CARGO_HOME: ./.cargo

      buildCommands:
        # Fetch only — don't compile. Developer will run 'cargo run' or 'cargo build'
        # via SSH after SSHing into the container. Pre-compiling wastes time because
        # the developer will immediately change code and recompile anyway.
        - cargo fetch

      # Deploy the entire working directory: source code, Cargo.toml/lock,
      # and the downloaded .cargo/registry. This gives the developer a ready-to-compile
      # workspace with all crates already cached locally.
      deployFiles:
        - ./

      cache:
        - .cargo/registry

    run:
      base: rust@stable
      # Ubuntu provides both glibc AND a richer toolset for interactive development:
      # apt-get, curl, git, etc. Alpine's minimalism is ideal for production but
      # frustrating when SSH'd in and needing to install debugging tools.
      os: ubuntu

      ports:
        - port: 3000
          httpSupport: true

      envVariables:
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}
        DB_NAME: db
        # Absolute path needed for interactive SSH sessions — the shell's working
        # directory may differ from /var/www, so a relative ./.cargo could resolve
        # to an unexpected location. Prod build used relative (correct for build steps);
        # runtime uses absolute (correct for SSH and cargo invocations).
        CARGO_HOME: /var/www/.cargo

      # The migration still runs via zsc execOnce before the container becomes idle.
      # This means when the developer SSHs in, the database schema is already set up
      # and the greetings table is seeded — they can start coding immediately.
      initCommands:
        - zsc execOnce ${ZEROPS_appVersionId} -- cargo run --bin migrate

      # zsc noop keeps the container alive without starting the application.
      # Zerops requires a long-running process. The developer starts the server
      # manually via SSH: cargo run --bin rust-hello-world
      start: zsc noop --silent
```
<!-- #ZEROPS_EXTRACT_END:integration-guide# -->
