# Rust Hello World Recipe App

<!-- start-fragment: intro -->
Actix Web application connected to PostgreSQL, with a health check endpoint at `/` that verifies database connectivity and queries migrated data. Used within [Rust Hello World recipe](https://app.zerops.io/recipes/rust-hello-world) for [Zerops](https://zerops.io) platform.
<!-- end-fragment: intro -->

**Full recipe page and deploy with one-click**

[![Deploy on Zerops](https://github.com/zeropsio/recipe-shared-assets/blob/main/deploy-button/light/deploy-button.svg)](https://app.zerops.io/recipes/rust-hello-world?environment=small-production)

![Rust cover](https://github.com/zeropsio/recipe-shared-assets/blob/main/covers/svg/cover-rust.svg)

<!-- start-fragment: integration-guide -->
## Integration Guide

### 1. Adding `zerops.yaml`
The main application configuration file you place at the root of your repository, it tells Zerops how to build, deploy and run your application.

```yaml
zerops:
  # Production setup — compile release binaries, deploy minimal footprint.
  # Matching rust@stable for build and run prevents glibc version mismatches.
  - setup: prod
    build:
      base: rust@stable
      os: ubuntu
      # Redirect Cargo registry into the project tree so Zerops can cache it.
      # build.envVariables persists across all build steps — not just one command.
      envVariables:
        CARGO_HOME: ./.cargo
      buildCommands:
        # --locked requires Cargo.lock in source; prevents silent dependency drift.
        # Builds both rust-hello-world and migrate binaries (all [[bin]] targets).
        - cargo build --release --locked
      deployFiles:
        # Both binaries ship to runtime — migrate runs in initCommands, not build time.
        - ./target/release/rust-hello-world
        - ./target/release/migrate
      cache:
        # Cache downloaded registry; compiled artifacts are version-specific, not worth caching.
        - .cargo/registry

    # Readiness check: verifies containers are ready before project balancer routes traffic.
    deploy:
      readinessCheck:
        httpGet:
          port: 3000
          path: /

    run:
      base: rust@stable
      # tokio-postgres links against glibc — ubuntu runtime matches the build environment.
      os: ubuntu

      # Run migration once per deploy. In initCommands — not buildCommands — so
      # migration and code deploy atomically. zsc execOnce prevents race conditions
      # when multiple containers start simultaneously.
      initCommands:
        - zsc execOnce ${appVersionId} -- ./target/release/migrate

      ports:
        - port: 3000
          httpSupport: true
      envVariables:
        # ${db_hostname} references the 'db' service — Zerops pattern: {hostname}_{key}.
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}
        DB_NAME: db
      start: ./target/release/rust-hello-world

  # Dev setup — deploy full source tree, developer runs the app via SSH.
  # cargo fetch downloads dependencies; developer compiles and runs interactively.
  - setup: dev
    build:
      base: rust@stable
      os: ubuntu
      envVariables:
        CARGO_HOME: ./.cargo
      buildCommands:
        # Only fetch dependencies — no compilation. Developer runs cargo build/run via SSH.
        - cargo fetch
      deployFiles:
        # Deploy entire source tree (includes downloaded registry) for SSH development.
        - ./
      cache:
        - .cargo/registry

    run:
      base: rust@stable
      # ubuntu provides apt-get and richer toolset for interactive SSH sessions.
      os: ubuntu

      initCommands:
        # cargo run --bin migrate compiles on first deploy (debug mode, one-time cost).
        # zsc execOnce ensures it runs exactly once even with multiple containers.
        - zsc execOnce ${appVersionId} -- cargo run --bin migrate

      ports:
        - port: 3000
          httpSupport: true
      envVariables:
        DB_HOST: ${db_hostname}
        DB_PORT: ${db_port}
        DB_USER: ${db_user}
        DB_PASS: ${db_password}
        DB_NAME: db
        # Absolute path — SSH sessions need CARGO_HOME to find the deployed registry.
        CARGO_HOME: /var/www/.cargo
      # Container stays idle — developer drives via SSH (cargo run, cargo watch, etc.).
      start: zsc noop --silent
```

### 2. Environment variables

Zerops injects all variables at runtime — no `.env` file needed. Variable names follow the `{hostname}_{key}` pattern based on the service hostname defined in your import.yaml.

| Variable | Source | Notes |
|----------|--------|-------|
| `DB_HOST` | `${db_hostname}` | Resolves to the `db` service hostname |
| `DB_PORT` | `${db_port}` | PostgreSQL default: 5432 |
| `DB_USER` | `${db_user}` | Auto-generated credentials |
| `DB_PASS` | `${db_password}` | Auto-generated credentials |
| `DB_NAME` | `db` | Static — matches the service hostname |

### 3. Migration

`src/bin/migrate.rs` creates the `greetings` table and seeds the initial row. It uses the same database driver (`tokio-postgres`) and environment variables as the main application. The migration is idempotent (`IF NOT EXISTS`, `ON CONFLICT DO NOTHING`) and runs via `zsc execOnce` in `initCommands` — once per deploy, regardless of container count.

### 4. `Cargo.lock`

Commit `Cargo.lock` to your repository. The production build uses `--locked` to ensure deterministic builds. If you're starting fresh: run `cargo fetch` locally (or push the dev setup first, then commit the generated lockfile from the runtime container).
<!-- end-fragment: integration-guide -->

---

## Your Knowledge Base

**You are expected to know**: build tools/package managers per language, deployFiles vs cache per runtime, base images for compiled vs interpreted languages, standard ports/frameworks, database drivers, idiomatic health checks, dependency file formats, tool differences (`cargo build --release --locked` vs `cargo fetch`), use case matching (prod=strict, dev=flexible), idiomatic migration patterns per language (raw SQL execution via the language's database driver).

**Apply this to create**:
- buildCommands for each setup's use case
- Migration script using the same database driver as the main app
- deployFiles based on what runtime needs (explicit names, not globs) — including migration scripts/binaries
- cache configuration for reusable artifacts (aligned with custom tool paths if any)
- start commands matching runtime environment
- initCommands with `zsc execOnce` wrapping the migration runner
- dependency installation matching dev vs prod context
- .gitignore covering actual artifact paths (especially when using custom tool HOME/cache dirs)

---

## Rules

1. Health check at `/` — not `/health`, not `/status`
2. Health check must actually test connections AND query migrated data — no fake "OK", no hardcoded greeting
3. Env vars follow `{hostname}_{key}` pattern — not `{service_type}`
4. Non-obvious decisions have WHY comments — three-tier system at 85% level
5. integration-guide fragment includes full zerops.yaml with comments
6. All 6 environments exist
7. Project names are unique — `{lang}-hello-world-{environment}`
8. Small Production has `minContainers: 2`
9. HA Production has `corePackage: SERIOUS` and dedicated CPUs
10. All databases/caches/storage have `priority: 10`
11. Every import.yaml is self-contained (Self-Containment Rule)
12. Custom tool HOME/cache paths set via `build.envVariables` — never inline shell prefixes or `prepareCommands` exports
13. Cache, deployFiles, and .gitignore agree on those same custom paths
14. **CRITICAL**: `minRam`, `minFreeRamGB`, and `cpuMode` MUST be nested under `verticalAutoscaling`.
15. **CRITICAL**: All migrations in `initCommands` MUST be wrapped in `zsc execOnce ${appVersionId}`.
16. **CRITICAL**: Migration binary must be included in `deployFiles` — it runs at runtime, not build time.
17. Migration SQL must be idempotent (`IF NOT EXISTS`, `ON CONFLICT DO NOTHING`) as defense-in-depth.
18. For compiled languages, migration binary must be built in `buildCommands` alongside the main app binary.
19. **CRITICAL**: `intro` fragment wraps ONLY description text — never the title, deploy button, or cover image.
