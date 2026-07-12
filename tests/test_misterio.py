"""Tests for the misterio core module: validation, config, aliases, orchestration, CLI."""

import os
import sys
from unittest.mock import ANY, patch

import pytest
from click.testing import CliRunner

from misterio.misterio import (
    load_misterio_config,
    misterio,
    misterio_cmd,
    process_role,
    verify_misterio_home,
)


# ---------------------------------------------------------------------------
# verify_misterio_home tests
# ---------------------------------------------------------------------------

class TestVerifyMisterioHome:
    """Tests for verify_misterio_home(home) — filesystem validation."""

    def test_valid_home(self, tmp_path):
        (tmp_path / "hosts").mkdir()
        (tmp_path / "roles").mkdir()
        verify_misterio_home(str(tmp_path))  # does not raise

    def test_missing_hosts(self, tmp_path):
        (tmp_path / "roles").mkdir()
        with pytest.raises(Exception, match="validation errors"):
            verify_misterio_home(str(tmp_path))

    def test_missing_roles(self, tmp_path):
        (tmp_path / "hosts").mkdir()
        with pytest.raises(Exception, match="validation errors"):
            verify_misterio_home(str(tmp_path))

    def test_both_missing(self, tmp_path):
        with pytest.raises(Exception, match="validation errors"):
            verify_misterio_home(str(tmp_path))

    def test_hosts_is_file_not_dir(self, tmp_path):
        (tmp_path / "hosts").write_text("not a directory")
        (tmp_path / "roles").mkdir()
        with pytest.raises(Exception, match="validation errors"):
            verify_misterio_home(str(tmp_path))


# ---------------------------------------------------------------------------
# load_misterio_config tests
# ---------------------------------------------------------------------------

class TestLoadMisterioConfig:
    """Tests for load_misterio_config(config_file_name, host_target)."""

    def test_missing_config_file_logs_notice(self, tmp_path, capsys):
        config_path = tmp_path / "nonexistent.toml"
        load_misterio_config(str(config_path), "somehost")
        captured = capsys.readouterr()
        assert "not defined" in captured.out

    def test_sets_docker_host_from_toml(self, tmp_path, monkeypatch):
        monkeypatch.delenv("DOCKER_HOST", raising=False)
        config_path = tmp_path / "misterio.toml"
        config_path.write_text('[host_x.docker]\nhost = "tcp://example.com:2375"\n')
        load_misterio_config(str(config_path), "host_x")
        assert os.environ["DOCKER_HOST"] == "tcp://example.com:2375"

    def test_sets_docker_context_from_toml(self, tmp_path, monkeypatch):
        monkeypatch.delenv("DOCKER_CONTEXT", raising=False)
        config_path = tmp_path / "misterio.toml"
        config_path.write_text('[host_x.docker]\ncontext = "orbstack"\n')
        load_misterio_config(str(config_path), "host_x")
        assert os.environ["DOCKER_CONTEXT"] == "orbstack"

    def test_nonexistent_host_key_is_noop(self, tmp_path, monkeypatch):
        monkeypatch.delenv("DOCKER_HOST", raising=False)
        monkeypatch.delenv("DOCKER_CONTEXT", raising=False)
        config_path = tmp_path / "misterio.toml"
        config_path.write_text('[other_host.docker]\nhost = "tcp://..."\n')
        load_misterio_config(str(config_path), "host_x")
        # Env vars should NOT be set for a missing key
        assert "DOCKER_HOST" not in os.environ
        assert "DOCKER_CONTEXT" not in os.environ

    def test_invalid_toml_propagates_error(self, tmp_path):
        config_path = tmp_path / "misterio.toml"
        config_path.write_text("this is not valid TOML {{{")
        import tomllib
        with pytest.raises(tomllib.TOMLDecodeError):
            load_misterio_config(str(config_path), "host_x")


# ---------------------------------------------------------------------------
# process_role tests
# ---------------------------------------------------------------------------

class TestProcessRole:
    """Tests for process_role() — alias expansion and delegation."""

    @pytest.fixture(autouse=True)
    def setup(self, mock_low_level_pr):
        self.mock_llp = mock_low_level_pr

    def test_normal_command_passes_through(self):
        process_role("/fake/home", "/fake/home/host/env.env", ["ps", "--tail", "5"])
        self.mock_llp.assert_called_once_with(
            "/fake/home", "/fake/home/host/env.env", ["ps", "--tail", "5"]
        )

    def test_rebuild_alias_expands_to_down_and_up(self):
        process_role("/fake/home", "/fake/home/host/env.env", ["@rebuild"])
        assert self.mock_llp.call_count == 2
        self.mock_llp.assert_any_call("/fake/home", "/fake/home/host/env.env", ["down"])
        self.mock_llp.assert_any_call(
            "/fake/home", "/fake/home/host/env.env", ["up", "--build", "-d"]
        )

    def test_upgrade_alias_expands_to_pull_down_up(self):
        process_role("/fake/home", "/fake/home/host/env.env", ["@upgrade"])
        assert self.mock_llp.call_count == 3
        self.mock_llp.assert_any_call("/fake/home", "/fake/home/host/env.env", ["pull"])
        self.mock_llp.assert_any_call("/fake/home", "/fake/home/host/env.env", ["down"])
        self.mock_llp.assert_any_call(
            "/fake/home", "/fake/home/host/env.env", ["up", "--build", "-d"]
        )

    def test_unknown_alias_raises(self):
        with pytest.raises(Exception, match="Unknown alias"):
            process_role("/fake/home", "/fake/home/host/env.env", ["@unknown"])
        self.mock_llp.assert_not_called()

    def test_single_char_not_treated_as_alias(self):
        """A single character (len < 2) does not enter alias processing."""
        process_role("/fake/home", "/fake/home/host/env.env", ["x"])
        self.mock_llp.assert_called_once_with(
            "/fake/home", "/fake/home/host/env.env", ["x"]
        )


# ---------------------------------------------------------------------------
# misterio_cmd tests
# ---------------------------------------------------------------------------

class TestMisterioCmd:
    """Tests for misterio_cmd() — the main orchestrator."""

    @pytest.fixture(autouse=True)
    def setup(self, mock_process_role, mock_load_config):
        self.mock_pr = mock_process_role
        self.mock_lc = mock_load_config

    def test_list_flag_prints_roles_and_exits(self, misterio_home):
        with pytest.raises(SystemExit) as exc_info:
            misterio_cmd(
                home=str(misterio_home),
                list_flag=True,
                misterio_host=None,
                single_role=None,
                docker_command=(),
            )
        assert exc_info.value.code == 0
        # process_role should never be called in list mode
        self.mock_pr.assert_not_called()

    def test_single_host_processes_all_roles(self, misterio_home):
        misterio_cmd(
            home=str(misterio_home),
            list_flag=False,
            misterio_host=("host_a",),
            single_role=None,
            docker_command=("up", "-d"),
        )
        # host_a has web.env and db.env — both should be processed
        assert self.mock_pr.call_count == 2

    def test_single_role_filter_limits_processing(self, misterio_home):
        misterio_cmd(
            home=str(misterio_home),
            list_flag=False,
            misterio_host=("host_a",),
            single_role="web",
            docker_command=("ps",),
        )
        # Only web.env matches — process_role called once
        assert self.mock_pr.call_count == 1
        call_args = self.mock_pr.call_args[0]
        assert call_args[1].endswith("web.env")

    def test_localhost_rejected(self, misterio_home):
        with pytest.raises(Exception, match="Use misterio.toml"):
            misterio_cmd(
                home=str(misterio_home),
                list_flag=False,
                misterio_host=("localhost",),
                single_role=None,
                docker_command=(),
            )

    def test_auto_discovers_hosts_from_filesystem(self, misterio_home):
        """When misterio_host is an empty tuple, hosts are discovered from
        the hosts/ directory, filtering out misterio.toml."""
        misterio_cmd(
            home=str(misterio_home),
            list_flag=False,
            misterio_host=(),  # empty → triggers auto-discovery
            single_role=None,
            docker_command=(),
        )
        # host_a has 2 roles, host_b has 1 role → 3 total process_role calls
        assert self.mock_pr.call_count == 3

    def test_calls_load_config_per_host(self, misterio_home):
        misterio_cmd(
            home=str(misterio_home),
            list_flag=False,
            misterio_host=("host_a",),
            single_role=None,
            docker_command=(),
        )
        expected_config_path = os.path.join(
            str(misterio_home), "hosts", "misterio.toml"
        )
        self.mock_lc.assert_called_once_with(expected_config_path, "host_a")


# ---------------------------------------------------------------------------
# misterio CLI entry point tests (CliRunner)
# ---------------------------------------------------------------------------

class TestMisterioCLI:
    """Tests for the misterio Click CLI command."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_help_shows_banner(self, runner):
        result = runner.invoke(misterio, ["--help"])
        assert result.exit_code == 0
        assert "M I S T E R I O" in result.output


    def test_list_with_valid_home(self, runner, misterio_home):
        result = runner.invoke(
            misterio, ["--home", str(misterio_home), "--list"]
        )
        assert result.exit_code == 0
        # Should list roles for discovered hosts
        assert "web.env" in result.output or "Roles for" in result.output
