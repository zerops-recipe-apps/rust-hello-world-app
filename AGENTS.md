# rust-hello-world-app

Actix-web + tokio-postgres API with a PostgreSQL-backed health endpoint and a separate `migrate` binary run via `zsc execOnce` — baseline Rust recipe on Zerops.

## Zerops service facts

- HTTP port: `8080`
- Siblings: `db` (PostgreSQL) — env: `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASS`, `DB_NAME`
- Runtime base: `rust@stable` (Ubuntu on dev)

## Zerops dev

`setup: dev` idles on `zsc noop --silent`; the agent starts the dev server.

- Dev command: `cargo run`
- In-container rebuild without deploy: `cargo build --release`

**All platform operations (start/stop/status/logs of the dev server, deploy, env / scaling / storage / domains) go through the Zerops development workflow via `zcp` MCP tools. Don't shell out to `zcli`.**

## Notes

- `CARGO_HOME` is redirected to `./.cargo` in the build so Zerops can cache the registry between builds; dev run overrides `CARGO_HOME=/var/www/.cargo` for interactive SSH sessions.
- Dev build uses `cargo fetch` only — the first `cargo run` compiles deps inside the container; `initCommands` runs migrations via `cargo run --bin migrate` under `zsc execOnce`.
