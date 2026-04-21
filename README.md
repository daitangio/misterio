# Misterio

Minimal multi-host `docker compose` orchestration, rewritten in Rust.

The model stays the same:

- `roles/<role>/` contains a compose project
- `hosts/<host>/<role>.env` enables that role on that host
- `misterio` walks hosts and roles, sets `DOCKER_HOST`, copies the selected env file to `.env`, and runs `docker compose`

## Build

```sh
cargo build
```

The crate exposes four binaries:

```sh
cargo run --bin misterio -- --help
cargo run --bin misterio-add -- --help
cargo run --bin misterio-mv -- --help
cargo run --bin misterio-rm -- --help
```

## Layout

```text
misterio_project/
├── hosts/
│   ├── misterio.toml
│   ├── alice/
│   │   └── elasticsearch.env
│   └── bob/
│       ├── elasticsearch.env
│       └── gitlab.env
├── roles/
│   ├── elasticsearch/
│   │   └── docker-compose.yml
│   └── gitlab/
│       └── docker-compose.yml
└── attic/
```

## Usage

Rebuild everything:

```sh
misterio --home ./misterio_project @rebuild
```

Run a normal compose command on every discovered role:

```sh
misterio --home ./misterio_project -- logs --tail 10
```

Restrict to one host:

```sh
misterio --home ./misterio_project -h alice -- ps
```

Restrict to one role:

```sh
misterio --home ./misterio_project -r elasticsearch ps
```

Create env files for new roles:

```sh
misterio-add --home ./misterio_project bob pgvector@1 pgvector@2
```

Move a role between hosts:

```sh
misterio-mv --home ./misterio_project gitlab alice bob
```

Remove a role and archive its env file:

```sh
misterio-rm --home ./misterio_project bob gitlab
```

## Host configuration

`hosts/misterio.toml` is optional. To keep the Rust rewrite simple, the parser only reads the subset Misterio actually uses today:

```toml
[alice.docker]
context = "orbstack"
host = "ssh://alice"
```

## Notes

- The Rust version keeps the original alias commands: `@rebuild` and `@upgrade`.
- `misterio-add` still creates the default `MISTERIO_*` variables.
- `localhost` is still rejected in favor of `misterio.toml`.
