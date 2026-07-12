## General rules

- Think before acting. Read existing files before writing code.
- Be concise in output but thorough in reasoning.
- Prefer editing over rewriting whole files.
- Do not re-read files you have already read.
- Test your code before declaring done.
- No sycophantic openers or closing fluff.
- Keep solutions simple and direct.
- The overall project aims to be very compact (*less is more* mantra)
- Always ignore the var/ folder: nothing interesting is there.

- At the end of LOG.md, create a work in progress log, where you note what you already did, what is missing. Always update LOG.md following the template. Always include the name of the model used

## Build & Develop

```sh
# Create virtualenv (Python >= 3.11 required)
python3 -m venv .venv && source .venv/bin/activate

# Install in editable mode
pip install -e .

# Run the CLI
misterio --help
misterio --home $PWD -- up -d
misterio -h somehost -- ps
```

## Lint & Format

```sh
pip install pylint black
pylint $(git ls-files '*.py')        # uses .pylintrc at repo root
black src/
```

CI runs `pylint` on push across Python 3.11–3.14 (see `.github/workflows/pylint.yml`).

## Publish

```sh
pip install build twine
python3 -m build
python3 -m twine upload dist/*
git tag <version>
```

Uses `flit_core` as the PEP 517 build backend (see `pyproject.toml`).

## Architecture

Misterio is a minimal, agent-less alternative to Ansible/SaltStack — it applies `docker compose` commands across remote hosts via SSH. The **controlling host** runs misterio; **target hosts** only need Docker Engine and SSH access.

### Source layout (`src/misterio/`)

| File | Purpose |
|---|---|
| `misterio.py` | Core CLI (`misterio` command via Click). Discovers hosts/roles, sets `DOCKER_HOST=ssh://<host>`, and runs `docker compose` per role. |
| `misterio_util.py` | Companion CLIs: `misterio-add`, `misterio-mv`, `misterio-rm`. Reuses `misterio_cmd()` from the core module. |
| `__init__.py` | Empty (package marker). |

### Runtime directory layout (`--home` / `$MISTERIO_HOME`)

```
<home>/
├── hosts/
│   ├── misterio.toml          # optional per-host Docker config (TOML, API v1.6)
│   ├── <hostname>/
│   │   └── <role>[@inst].env  # one env file per role per host
│   └── <otherhost>/
│       └── ...
├── roles/
│   └── <rolename>/
│       └── docker-compose.yml # or docker-compose.yaml
└── attic/                     # auto-created; stores removed role env files
```

### Key concepts

- **Role**: a directory under `roles/` containing a `docker-compose.yml`. The role name equals the directory name.
- **Host**: a directory under `hosts/` named after a reachable SSH host. Misterio sets `DOCKER_HOST=ssh://<hostname>` before running compose commands.
- **Env file**: `<role>[@instance].env` — copied to `.env` inside the role directory before `docker compose` runs. The `@instance` suffix enables multiple instances of the same role on one host.
- **`misterio.toml`**: optional, keyed by hostname. Allows overriding `DOCKER_CONTEXT` and `DOCKER_HOST` per host (useful for OrbStack or non-standard Docker setups). Uses Python's `tomllib` (Python >= 3.11).
- **Alias commands**: `@rebuild` (down → up --build -d) and `@upgrade` (pull → down → up --build -d) are handled in `process_role()` before shelling out.
- **SSH**: misterio uses `ssh://<hostname>` as the Docker host. Customize SSH behavior via `~/.ssh/config`. It prefers `rsync` for file transfer (falls back to `scp`).

### Control flow

1. `misterio` entry point parses CLI args (Click).
2. `misterio_cmd()` validates `--home` has `hosts/` and `roles/` dirs.
3. Iterates host directories, sets `DOCKER_HOST`, loads optional `misterio.toml` config.
4. For each `.env` file in the host dir (optionally filtered by `--single-role`), calls `process_role()`.
5. `process_role()` handles alias expansion, then `low_level_pr()` copies the env file and runs `docker compose <args>` in the role directory.

### Tests

```sh
pip install -e ".[test]"
python -m pytest tests/ -v --tb=short
```

Tests live in `tests/` and use pytest with stdlib `unittest.mock`. Pure functions (`write_prop`, `determine_instance_name`, `determine_fixed_port`) are tested directly. CLI commands are tested via Click's `CliRunner` with an isolated filesystem. The subprocess/Docker boundary is mocked so no Docker or SSH is needed to run the test suite.
