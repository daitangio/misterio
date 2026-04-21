# Repository Guidelines

## Project Structure & Module Organization
Core CLI entrypoints live in `cmd/` with one binary per command: `cmd/misterio`, `cmd/misterio-add`, `cmd/misterio-mv`, and `cmd/misterio-rm`. Shared application logic is in `internal/misterio/`, and the lightweight TOML parser for `hosts/misterio.toml` is in `internal/config/`.

Example infrastructure lives under `roles/`, with one directory per Docker Compose stack, such as `roles/pgvector/` or `roles/watchtower/`. Host-specific role assignments live under `hosts/`, usually as `<role>[@instance].env` files. Supporting docs and scripts are in `etc/`, `sh/`, and top-level Markdown files. The legacy shell implementation is kept in `old_sh_version/`.

## Build, Test, and Development Commands
Set up a local environment with:

```sh
go build ./...
go test ./...
```

Useful commands:

- `misterio --help`: show CLI usage.
- `misterio --home ./demo --list`: list configured roles per host.
- `misterio --home ./demo @rebuild`: rebuild and restart all configured roles.
- `go test ./...`: run the automated test suite.
- `go build ./...`: compile every command.
- `gofmt -w ./cmd ./internal`: format Go sources.
- `go vet ./...`: run static checks before opening a PR.

## Coding Style & Naming Conventions
Target Go is the version declared in `go.mod`. Keep packages small, prefer standard-library dependencies, and place reusable logic in `internal/` rather than in `main` packages. Use `camelCase` for unexported identifiers, `PascalCase` for exported types, and keep role directories lowercase with host env files named `<role>.env` or `<role>@<instance>.env`.

Always run `gofmt` on edited files. Keep command-line behavior backward-compatible unless the PR clearly documents an intentional change.

## Testing Guidelines
Add table-driven tests for helper logic and config parsing in the same package as the code under test. Before opening a PR, run `go test ./...` and exercise the CLI path you changed against a sample `hosts/` and `roles/` layout, for example `misterio-add testhost pgvector@1`.

## Commit & Pull Request Guidelines
Recent commits use short, imperative subjects such as `Support for test container` and `Build 0.1.5`. Keep commit titles concise and focused on one change.

PRs should explain the operational impact, note any required host or role layout changes, and include command examples when CLI behavior changes. Add screenshots only when documentation or terminal output clarity benefits from them.
