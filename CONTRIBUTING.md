# Contributing to M-Chrono

## Setup

```bash
# Prerequisites: Rust stable, Node.js >= 20, bun
bun install
bun run tauri dev
```

Linux requires additional system packages — see the CI workflow for the full list.

## Running tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # Rust
bun run test --run                                  # Svelte / Vitest
bun run check                                       # TypeScript / Svelte types
```

## Pull requests

- Keep PRs focused — one thing per PR
- All tests must pass before merging
- Use the PR template; link to the relevant issue

## Reporting bugs

Use the Bug Report issue template. Include OS version, app version, and relevant log lines from `<app_data_dir>/logs/`.

## License

By contributing you agree that your changes will be released under the [MIT License](LICENSE).
