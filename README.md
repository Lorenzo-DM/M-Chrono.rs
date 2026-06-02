# Trail Race Timing (Desktop)

Tauri 2 + Svelte 5 desktop chronometry app for trail running races.
Offline-first SQLite persistence, OIDC device-code auth via Zitadel,
bidirectional cloud sync with dual-operator dedup, XLSX export.

See `docs/superpowers/specs/2026-06-01-trail-race-timing-desktop-design.md`
for the full design and `docs/superpowers/plans/2026-06-01-trail-race-timing-desktop.md`
for the implementation plan.

## Requirements

- Rust stable (>= 1.78)
- Node.js >= 20
- bun (package manager; not pnpm/npm)
- macOS, Linux, or Windows
- Zitadel instance configured with a Native client supporting Device
  Authorization Grant; refresh-token TTL >= 30 days

## Development

```bash
bun install
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

Artifacts under `src-tauri/target/release/bundle/`.

## Configuration

First launch writes `config.json` under the OS app data dir:

- macOS: `~/Library/Application Support/com.tauri.dev/`
- Linux: `~/.local/share/com.tauri.dev/`
- Windows: `%APPDATA%\com.tauri.dev\`

Configure via Settings UI or directly in `config.json`:

- `oidc_issuer_url` — e.g. `https://example.zitadel.cloud`
- `oidc_client_id` — Native client ID
- `oidc_scopes` — `openid profile email offline_access ...`
- `api_base_url` — race API root
- `operator_id` — unique label per desktop (e.g. `PC-A`, `PC-B`)
- `sync_interval_secs` — background sync cadence (default 10)
- `dedup_window_ms` — sliding window for duplicate grouping (default 2000)
- `dedup_warn_delta_ms` — flag threshold within group (default 500)

Refresh token is stored in the OS keychain. Access tokens live in
memory only.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
bun run test --run
```

## Zitadel setup

1. Create a Native client in your project; enable Device Authorization
   Grant.
2. In Token Settings: Refresh Token Expiration >= 30 days; Idle
   Expiration >= 30 days (or disabled).
3. Add a custom scope or audience for the race API; mirror it in
   `oidc_scopes`.
4. Copy the client_id into `oidc_client_id`.

## Logs

Daily-rotated log files are written to `<app_data_dir>/logs/race.log.<date>`.
Filter via `RUST_LOG`, e.g. `RUST_LOG=trailtrace_stopwatch_lib=debug`.

## Smoke tests

See `scripts/smoke-finish.mjs` for documented long-run procedure.
