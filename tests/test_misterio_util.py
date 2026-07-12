"""Tests for misterio_util: pure functions and CLI commands (misterio-add, misterio-rm, misterio-mv)."""

import io
import os
from unittest.mock import ANY, call, patch

import pytest
from click.testing import CliRunner

from misterio.misterio_util import (
    determine_fixed_port,
    determine_instance_name,
    misterio_add,
    misterio_mv,
    misterio_rm,
    write_prop,
)


# ---------------------------------------------------------------------------
# write_prop tests
# ---------------------------------------------------------------------------

class TestWriteProp:
    """Tests for write_prop(key, value, f) — pure function with injected file handle."""

    def test_simple_string_value(self):
        """Simple string without spaces: KEY=value (no quotes)."""
        buf = io.StringIO()
        write_prop("my_key", "simple", buf)
        assert buf.getvalue() == "MY_KEY=simple\n"

    def test_value_with_spaces_gets_quoted(self):
        """String containing spaces gets double-quoted."""
        buf = io.StringIO()
        write_prop("my_key", "has space", buf)
        assert buf.getvalue() == 'MY_KEY="has space"\n'

    def test_non_string_value_gets_quoted(self):
        """Non-string values (e.g. int) are stringified and quoted."""
        buf = io.StringIO()
        write_prop("port", 8080, buf)
        assert buf.getvalue() == 'PORT="8080"\n'

    def test_empty_string_value_no_quotes(self):
        """Empty string: no space, no quotes."""
        buf = io.StringIO()
        write_prop("empty", "", buf)
        assert buf.getvalue() == "EMPTY=\n"

    def test_output_ends_with_newline(self):
        """Every write_prop call appends exactly one newline."""
        buf = io.StringIO()
        write_prop("key", "val", buf)
        assert buf.getvalue().endswith("\n")
        # Single newline, not double
        assert not buf.getvalue().endswith("\n\n")


# ---------------------------------------------------------------------------
# determine_instance_name tests
# ---------------------------------------------------------------------------

class TestDetermineInstanceName:
    """Tests for determine_instance_name(role) — pure string function."""

    def test_simple_name(self):
        assert determine_instance_name("postgres") == "postgres"

    def test_with_instance_suffix(self):
        assert determine_instance_name("postgres@1") == "postgres_1"

    def test_uppercase_input_lowered(self):
        assert determine_instance_name("Postgres@Prod") == "postgres_prod"

    def test_hyphens_preserved(self):
        assert determine_instance_name("elastic-search@instance_2") == "elastic-search_instance_2"

    def test_no_at_sign_just_lowers(self):
        assert determine_instance_name("MyRole") == "myrole"


# ---------------------------------------------------------------------------
# determine_fixed_port tests
# ---------------------------------------------------------------------------

class TestDetermineFixedPort:
    """Tests for determine_fixed_port(role, base_port=7000) — pure function."""

    def test_simple_name_default_base(self):
        # len("postgres") = 8, base_port=7000 → 7008
        assert determine_fixed_port("postgres") == 7008

    def test_with_numeric_instance(self):
        # len("postgres@1")=10, idx=int(str(int("1",36))[0:2])=int("1")=1 → 7000+10+1=7011
        assert determine_fixed_port("postgres@1") == 7011

    def test_custom_base_port(self):
        # len("web")=3, base_port=8000 → 8003
        assert determine_fixed_port("web", base_port=8000) == 8003

    def test_letter_instance_suffix_base36(self):
        # len("test@a")=6, int("a",36)=10, str(10)[0:2]="10" → idx=10 → 7000+6+10=7016
        assert determine_fixed_port("test@a") == 7016

    def test_multi_char_instance_truncated(self):
        # len("x@ff")=4, int("ff",36)=555, str(555)[0:2]="55" → idx=55 → 7000+4+55=7059
        assert determine_fixed_port("x@ff") == 7059


# ---------------------------------------------------------------------------
# misterio_add CLI tests
# ---------------------------------------------------------------------------

class TestMisterioAdd:
    """Tests for the misterio-add CLI command."""

    @pytest.fixture(autouse=True)
    def setup(self, mock_misterio_cmd, monkeypatch):
        """Mock misterio_cmd and stabilize env-dependent values."""
        self.mock_cmd = mock_misterio_cmd
        monkeypatch.setenv("USER", "testuser")
        self.monkeypatch = monkeypatch

    def invoke(self, tmp_path, *args):
        """Helper: invoke misterio-add with --home pointing to tmp_path."""
        runner = CliRunner()
        return runner.invoke(misterio_add, ["--home", str(tmp_path)] + list(args))

    def test_creates_env_file(self, tmp_path):
        result = self.invoke(tmp_path, "myhost", "myrole")
        assert result.exit_code == 0
        env_file = tmp_path / "hosts" / "myhost" / "myrole.env"
        assert env_file.exists()

    def test_role_already_exists_is_rejected(self, tmp_path):
        self.invoke(tmp_path, "myhost", "myrole")
        result = self.invoke(tmp_path, "myhost", "myrole")
        assert result.exit_code == 0
        assert "FATAL" in result.output

    def test_creates_multiple_roles(self, tmp_path):
        result = self.invoke(tmp_path, "myhost", "r1", "r2")
        assert result.exit_code == 0
        assert (tmp_path / "hosts" / "myhost" / "r1.env").exists()
        assert (tmp_path / "hosts" / "myhost" / "r2.env").exists()

    def test_writes_correct_properties(self, tmp_path):
        self.invoke(tmp_path, "myhost", "myrole")
        content = (tmp_path / "hosts" / "myhost" / "myrole.env").read_text()
        assert "MISTERIO_CREATION_USER" in content
        assert "MISTERIO_CREATION_DATE" in content
        assert "MYROLE_HOME" in content
        assert "MISTERIO_MAGIPORT" in content

    def test_build_flag_calls_misterio_cmd(self, tmp_path):
        self.invoke(tmp_path, "--build", "myhost", "myrole")
        self.mock_cmd.assert_called_once_with(
            home=str(tmp_path),
            list_flag=None,
            misterio_host=["myhost"],
            single_role="myrole",
            docker_command=["build"],
        )


# ---------------------------------------------------------------------------
# misterio_rm CLI tests
# ---------------------------------------------------------------------------

class TestMisterioRm:
    """Tests for the misterio-rm CLI command."""

    @pytest.fixture(autouse=True)
    def setup(self, mock_misterio_cmd):
        self.mock_cmd = mock_misterio_cmd

    def invoke(self, tmp_path, *args):
        runner = CliRunner()
        return runner.invoke(misterio_rm, ["--home", str(tmp_path)] + list(args))

    def _create_env(self, tmp_path, host, role, content="KEY=val\n"):
        env_file = tmp_path / "hosts" / host / f"{role}.env"
        env_file.parent.mkdir(parents=True, exist_ok=True)
        env_file.write_text(content)
        return env_file

    def test_moves_env_to_attic(self, tmp_path):
        self._create_env(tmp_path, "myhost", "myrole")
        self.invoke(tmp_path, "myhost", "myrole")
        # Env file should be gone from hosts/
        assert not (tmp_path / "hosts" / "myhost" / "myrole.env").exists()
        # And present in attic/
        assert (tmp_path / "attic" / "myhost" / "myrole.env").exists()

    def test_removes_multiple_roles(self, tmp_path):
        self._create_env(tmp_path, "myhost", "r1")
        self._create_env(tmp_path, "myhost", "r2")
        self.invoke(tmp_path, "myhost", "r1", "r2")
        assert not (tmp_path / "hosts" / "myhost" / "r1.env").exists()
        assert not (tmp_path / "hosts" / "myhost" / "r2.env").exists()
        assert (tmp_path / "attic" / "myhost" / "r1.env").exists()
        assert (tmp_path / "attic" / "myhost" / "r2.env").exists()

    def test_preserves_file_content(self, tmp_path):
        secret = "SECRET_TOKEN=abc123xyz\nPORT=9999\n"
        self._create_env(tmp_path, "myhost", "myrole", content=secret)
        self.invoke(tmp_path, "myhost", "myrole")
        attic_content = (tmp_path / "attic" / "myhost" / "myrole.env").read_text()
        assert attic_content == secret


# ---------------------------------------------------------------------------
# misterio_mv CLI tests
# ---------------------------------------------------------------------------

class TestMisterioMv:
    """Tests for the misterio-mv CLI command."""

    @pytest.fixture(autouse=True)
    def setup(self, mock_misterio_cmd):
        self.mock_cmd = mock_misterio_cmd

    def invoke(self, tmp_path, *args):
        runner = CliRunner()
        return runner.invoke(misterio_mv, ["--home", str(tmp_path)] + list(args))

    def _create_env(self, tmp_path, host, role, content="KEY=val\n"):
        env_file = tmp_path / "hosts" / host / f"{role}.env"
        env_file.parent.mkdir(parents=True, exist_ok=True)
        env_file.write_text(content)
        return env_file

    def test_moves_file_between_hosts(self, tmp_path):
        self._create_env(tmp_path, "host_a", "myrole")
        # Destination directory must exist for shutil.move to succeed
        (tmp_path / "hosts" / "host_b").mkdir(parents=True, exist_ok=True)
        result = self.invoke(tmp_path, "myrole", "host_a", "host_b")
        assert result.exit_code == 0
        assert not (tmp_path / "hosts" / "host_a" / "myrole.env").exists()
        assert (tmp_path / "hosts" / "host_b" / "myrole.env").exists()

    def test_preserves_content_after_move(self, tmp_path):
        secret = "DB_PASSWORD=s3cret\n"
        self._create_env(tmp_path, "host_a", "myrole", content=secret)
        (tmp_path / "hosts" / "host_b").mkdir(parents=True, exist_ok=True)
        self.invoke(tmp_path, "myrole", "host_a", "host_b")
        moved_content = (tmp_path / "hosts" / "host_b" / "myrole.env").read_text()
        assert moved_content == secret
