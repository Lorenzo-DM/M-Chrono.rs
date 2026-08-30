# M-Chrono

[![CI](https://github.com/Lorenzo-DM/M-Chrono.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Lorenzo-DM/M-Chrono.rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A trail race finish line, timed by two volunteers on two laptops, with no
mobile signal. The usual answer is two paper logs reconciled by hand that
night, hoping the bib numbers and times line up.

M-Chrono runs fully offline with zero configuration: every timing event is
captured locally to SQLite on each laptop, so the app works with no network
and no setup. When a laptop later gets connectivity, it can sync to a shared
race API and the two logs merge automatically, with duplicate finishes
flagged for review — but that sync is optional, not something the app needs
in order to function.

Built with Tauri 2, Svelte 5, and Rust.

## Screenshots

![Timing view: the 21K and 40K courses timed side by side, each with its own running clock and a queue of captured finishes awaiting bib assignment](docs/screenshots/timing.png)

Two courses timed in parallel. Each lane keeps its own monotonic clock; finishes
are captured first and matched to a bib afterwards, so the operator never has to
type while runners are crossing.

## Features

- **Precise timing** — start/finish capture with a monotonic clock, multiple
  timing lanes, and per-athlete splits.
- **Offline-first** — local SQLite store; no data loss without network.
- **Dual-operator sync (optional)** — two desktops time the same race; when
  cloud sync is enabled, events merge automatically with a sliding-window
  dedup that flags suspicious deltas for manual review.
- **Bib management** — assign, reassign, and import athlete/bib lists.
- **Checkpoints** — intermediate timing points along the course.
- **Export** — XLSX and CSV results.
- **Import** — athlete rosters from spreadsheets (XLSX/CSV).
- **Auth (optional)** — OIDC device-code flow against any standards-compliant
  provider; tokens in the OS keychain. Only used if cloud sync is enabled.
- **i18n** — Italian (default) and English, switchable at runtime.

## Download

Pre-built binaries are on the
[latest release](https://github.com/Lorenzo-DM/M-Chrono.rs/releases/latest):

- macOS — universal `.dmg`, or `.app.tar.gz`
- Linux — `.AppImage`, `.deb`, or `.rpm`
- Windows — `.msi`, or `.exe` (NSIS installer)

No account or configuration is required to run the app.

## Requirements

To build from source (see [Download](#download) above for pre-built binaries):

- Rust stable (>= 1.78)
- Node.js >= 20
- [bun](https://bun.sh) (package manager; not pnpm/npm)
- macOS, Linux, or Windows

## Development

```bash
bun install
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

Artifacts land under `src-tauri/target/release/bundle/`.

## Architecture

```
src/                  Svelte 5 frontend (UI, stores, i18n)
  lib/components/      Race setup, workspace, results, export, settings
  lib/i18n/            Locale system (it, en)
src-tauri/src/         Rust backend
  timer/               Monotonic clock + timing events
  db/                  SQLite repo + migrations
  auth/                OIDC discovery, device-code + refresh, keychain store
  sync/                Pull/push + dedup against the race API
  api/                 Authenticated HTTP client
  export/              XLSX + CSV writers
  import/              Roster import
```

## Configuration

First launch writes `config.json` under the OS app data dir
(`com.mchrono.app`):

- macOS: `~/Library/Application Support/com.mchrono.app/`
- Linux: `~/.local/share/com.mchrono.app/`
- Windows: `%APPDATA%\com.mchrono.app\`

Configure via the Settings UI or directly in `config.json`. `operator_id`
applies regardless of sync; the rest only matters if you enable cloud sync
(see below).

| Key                   | Meaning                                              | Default |
| --------------------- | ---------------------------------------------------- | ------- |
| `oidc_issuer_url`     | OIDC issuer, e.g. `https://idp.example.com`          | —       |
| `oidc_client_id`      | Native client ID                                     | —       |
| `oidc_scopes`         | `openid profile email offline_access ...`            | —       |
| `api_base_url`        | race API root                                        | —       |
| `operator_id`         | unique label per desktop (e.g. `PC-A`, `PC-B`)       | —       |
| `sync_enabled`        | enable background cloud sync                          | `false` |
| `sync_interval_secs`  | background sync cadence                              | `10`    |
| `dedup_window_ms`     | sliding window for duplicate grouping                | `2000`  |
| `dedup_warn_delta_ms` | flag threshold within a dedup group                  | `500`   |

The refresh token is stored in the OS keychain. Access tokens live in memory
only.

## Cloud sync (optional)

Cloud sync is disabled by default (`sync_enabled: false`) and the app is
fully functional without it. Enable it only if you want two laptops' timing
logs to merge automatically through a shared race API instead of being
reconciled by hand.

Endpoints are resolved at runtime from the provider's discovery document at
`<oidc_issuer_url>/.well-known/openid-configuration`, so no provider-specific
URLs are hardcoded. Any provider that publishes a `device_authorization_endpoint`
there works — Keycloak, Auth0, Okta, Entra ID, and others.

1. Create a public (native) client and enable the Device Authorization Grant.
2. Enable refresh tokens and set their lifetime to cover a race weekend
   (>= 30 days recommended, with idle expiration disabled or equally long).
3. Add the scope or audience your race API expects; mirror it in `oidc_scopes`
   alongside `offline_access`.
4. Copy the client ID into `oidc_client_id` and the issuer URL into
   `oidc_issuer_url`.
5. Set `sync_enabled: true` in `config.json` (or via the Settings UI).

## Internationalization

UI strings live in `src/lib/i18n/locales/` (`it.ts`, `en.ts`). The locale is
resolved from `localStorage`, falling back to the system language, defaulting to
Italian. Switch languages at runtime from the Settings page.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # Rust backend
bun run test --run                                # Svelte frontend (vitest)
```

## Logs

Daily-rotated log files are written to `<app_data_dir>/logs/race.log.<date>`.
Filter via `RUST_LOG`, e.g. `RUST_LOG=m_chrono_lib=debug`.
