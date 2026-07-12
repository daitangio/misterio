"""Shared fixtures for misterio tests."""

import os
from pathlib import Path
from unittest.mock import patch

import pytest
from click.testing import CliRunner


@pytest.fixture
def runner():
    """Provide a fresh Click CliRunner for CLI tests."""
    return CliRunner()


@pytest.fixture
def misterio_home(tmp_path):
    """Create a minimal valid misterio home directory.

    Layout:
        <tmp_path>/
        ├── roles/                  (empty but present)
        ├── hosts/
        │   ├── misterio.toml       (must be filtered from host discovery)
        │   ├── host_a/
        │   │   ├── web.env
        │   │   └── db.env
        │   └── host_b/
        │       └── cache.env
    """
    (tmp_path / "roles").mkdir()
    hosts = tmp_path / "hosts"
    hosts.mkdir()

    (hosts / "host_a").mkdir()
    (hosts / "host_a" / "web.env").write_text("COMPOSE_PROJECT_NAME=web\n")
    (hosts / "host_a" / "db.env").write_text("COMPOSE_PROJECT_NAME=db\n")

    (hosts / "host_b").mkdir()
    (hosts / "host_b" / "cache.env").write_text("COMPOSE_PROJECT_NAME=cache\n")

    # misterio.toml must be filtered out during host auto-discovery
    (hosts / "misterio.toml").write_text("")

    return tmp_path


@pytest.fixture
def mock_low_level_pr():
    """Prevent low_level_pr from shelling out to Docker."""
    with patch("misterio.misterio.low_level_pr") as mock:
        yield mock


@pytest.fixture
def mock_process_role():
    """Prevent process_role from delegating to low_level_pr."""
    with patch("misterio.misterio.process_role") as mock:
        yield mock


@pytest.fixture
def mock_load_config():
    """Prevent load_misterio_config from reading TOML and mutating env vars."""
    with patch("misterio.misterio.load_misterio_config") as mock:
        yield mock


@pytest.fixture
def mock_misterio_cmd():
    """Prevent util CLIs from calling misterio_cmd (which would chdir/subprocess).

    Patches at misterio.misterio_util.misterio_cmd because the import is:
        from .misterio import misterio_cmd
    """
    with patch("misterio.misterio_util.misterio_cmd") as mock:
        yield mock


@pytest.fixture(autouse=True)
def cleanup_docker_env(monkeypatch):
    """Ensure DOCKER_HOST and DOCKER_CONTEXT are clean for every test.

    Multiple functions mutate these env vars: load_misterio_config sets them,
    misterio_cmd sets DOCKER_HOST=ssh://<hostname>.
    """
    monkeypatch.delenv("DOCKER_HOST", raising=False)
    monkeypatch.delenv("DOCKER_CONTEXT", raising=False)
