# Repository Guidelines

## Project Structure & Module Organization
`src/lib.rs` is the library entry point and wires together the internal modules in `src/app.rs`, `src/cli.rs`, and `src/config.rs`. CLI binaries live in `src/bin/` as `misterio.rs`, `misterio-add.rs`, `misterio-mv.rs`, and `misterio-rm.rs`.

Runtime fixture data lives outside `src/`: `roles/` holds example compose projects, `hosts/` holds host config and per-role `.env` files, and `old_sh_version/` preserves the legacy shell implementation for reference. Current Rust unit tests live in `src/tests.rs`; there is no top-level `tests/` directory yet.

## Build, Test, and Development Commands
Use Cargo for all local work:

- `cargo build` builds the library and all binaries.
- `cargo check --all-targets` matches the GitHub Actions validation step.
- `cargo test` runs the unit tests in `src/tests.rs`.
- `cargo run --bin misterio -- --help` checks the main CLI.
- `cargo run --bin misterio-add -- --help` verifies one of the helper binaries; swap in `misterio-mv` or `misterio-rm` as needed.

## Coding Style & Naming Conventions
Follow standard Rust formatting with 4-space indentation and run `cargo fmt` before opening a PR. Prefer small functions, explicit error messages with context, and `snake_case` for functions, modules, and local variables. Keep CLI-facing names aligned with the existing command set and file names, for example `misterio-add` in `src/bin/misterio-add.rs`.

## Testing Guidelines
Add focused unit tests beside the current suite in `src/tests.rs`, especially for parsing, host config handling, and role-selection behavior. Name tests after the behavior they lock down, such as `role_matching_is_exact_for_role_or_instance`. Run `cargo test` locally before pushing; if you change argument parsing or command wiring, also run the affected `cargo run --bin ... -- --help` command.

## Commit & Pull Request Guidelines
Recent history favors short, imperative commit subjects like `Fix help text` or `Support for test container`; use that style and keep one logical change per commit. PRs should explain the user-visible behavior change, note any updates under `hosts/` or `roles/`, and include the commands you ran to validate the change. Screenshots are unnecessary unless documentation or rendered output changes.
