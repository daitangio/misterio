import os
import sys
import shutil
import subprocess
import tomllib
from typing import List, Optional

import click


class MisterioError(Exception):
    """Base exception for Misterio errors."""

    pass


class MisterioConfigError(MisterioError):
    """Configuration-related error."""

    pass


class MisterioValidationError(MisterioError):
    """Validation error for Misterio home directory or inputs."""

    pass


def get_version() -> str:
    """Read version from pyproject.toml."""
    pyproject_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "..", "pyproject.toml"
    )
    try:
        with open(pyproject_path, "rb") as f:
            config = tomllib.load(f)
            return config["project"]["version"]
    except (FileNotFoundError, KeyError, tomllib.TomlDecodeError):
        return "?.?.?"


def process_role(home: str, env_full_path: str, docker_command: List[str]) -> None:
    """Process role command, expanding aliases if needed."""
    # print(docker_command)
    if (
        len(docker_command) == 1
        and len(docker_command[0]) >= 2
        and docker_command[0][0] == "@"
    ):
        # Process alias
        aliascmd = docker_command[0]
        if aliascmd == "@rebuild":
            # rebuild require two command to run:
            low_level_pr(home, env_full_path, ["down"])
            low_level_pr(home, env_full_path, ["up", "--build", "-d"])
        elif aliascmd == "@upgrade":
            low_level_pr(home, env_full_path, ["pull"])
            low_level_pr(home, env_full_path, ["down"])
            low_level_pr(home, env_full_path, ["up", "--build", "-d"])
        else:
            raise MisterioError(f"Unknown alias: {aliascmd}")
    else:
        low_level_pr(home, env_full_path, docker_command)


def low_level_pr(home: str, env_full_path: str, docker_command: List[str]) -> None:
    env_file: str = os.path.basename(env_full_path)
    role_name: str = (
        env_file.split("@")[0] if "@" in env_file else env_file.split(".env")[0]
    )
    role_dir: str = os.path.join(home, "roles", role_name)

    if not os.path.isdir(role_dir):
        raise MisterioValidationError(f"Role directory not found: {role_dir}")

    if not os.path.exists(env_full_path):
        raise MisterioValidationError(f"Env file not found: {env_full_path}")

    full_command = ["docker", "compose"]
    full_command.extend(docker_command)
    docker_host = os.getenv("DOCKER_HOST", "")
    full_command_log=" ".join(full_command)
    print(f"==== {docker_host} {role_name} \t-> {full_command_log}")
    shutil.copyfile(env_full_path, os.path.join(role_dir, ".env"))
    try:
        subprocess.run(full_command, check=True, cwd=role_dir)
    except subprocess.CalledProcessError as e:
        raise MisterioError(
            f"{docker_host}::{role_name} Failed with return code {e.returncode}"
        ) from e



def verify_misterio_home(home: str) -> None:
    error_count: int = 0
    for d in ["hosts", "roles"]:
        path: str = os.path.join(home, d)
        if not os.path.isdir(path):
            print(f"FATAL: Missed required directory {path}")
            error_count += 1
    if error_count > 0:
        raise MisterioValidationError(f"home dir has {error_count} validation errors")


def load_misterio_config(config_file_name: str, host_target: str) -> None:
    if os.path.exists(config_file_name):
        with open(config_file_name, "rb") as config_file:
            main_config: dict = tomllib.load(config_file).get(host_target, {})
            # Fast exit
            if len(main_config) == 0:
                return
            print(f"Config for {host_target}: {main_config}")
            if "docker" in main_config:
                docker_config: dict = main_config["docker"]
                if "context" in docker_config:
                    context = docker_config["context"]
                    os.environ["DOCKER_CONTEXT"] = context
                if "host" in docker_config:
                    docker_host = docker_config["host"]
                    os.environ["DOCKER_HOST"] = docker_host
    else:
        print(f"{config_file_name} not defined (see documentation for happy features)")


@click.command("misterio")
@click.option(
    "--home",
    envvar="MISTERIO_HOME",
    # default=lambda: os.getenv("PWD", ""),
    default=os.getenv("PWD", ""),
    help="Home of hosts and roles folders. Can be set with MISTERIO_HOME",
)
@click.option(
    "--misterio-host",
    "-h",
    multiple=True,
    default=None,
    help="Default to single hostname restriction. Can be overriden also with MISTERIO_HOST.",
)
@click.option(
    "--list/--no-list",
    "list_flag",
    help="List roles and exits",
    default=False,
)
@click.option(
    "--single-role",
    "-r",
    envvar="MISTERIO_SINGLE_ROLE",
    default=None,
    help="Process just one role",
)
@click.version_option(version=get_version())
@click.argument("docker_command", nargs=-1, type=str)
def misterio(
    home: str,
    list_flag: bool,
    misterio_host: Optional[List[str]],
    single_role: Optional[str],
    docker_command: List[str],
) -> None:
    """M I S T E R I O
    
    docker stateless compose-based alternative to K8s/Ansible/SaltStack.
    
    By default the system will scan all the hostname inside

        $MISTERIO_HOME/hosts/

        and connect to every of them

        Verify logs of all services:

        misterio -- logs --tail 5

        Verify logs of all services to just one server:

            misterio -h wonderboy -- logs --tail 5

        Verify clustered elastic-service on all nodes:

            misterio --single-role elastic-service ps

        Verify elastic service on just two nodes, named wonderboy and adam:

            misterio -h wonderboy -h adam --single-role elastic-service ps

        // Special Internal Commands //

        * @rebuild make a compose down/up to ensure everything is configured properly

        * @upgrade pull the images again, and then run @rebuild, with minimal disruption

    """
    return misterio_cmd(home, list_flag, misterio_host, single_role, docker_command)


def misterio_cmd(
    home: str,
    list_flag: bool,
    misterio_host: Optional[List[str]],
    single_role: Optional[str],
    docker_command: List[str],
) -> None:
    verify_misterio_home(home)
    if misterio_host is None or len(misterio_host) == 0:
        misterio_host_list: List[str] = [
            e
            for e in sorted(os.listdir(os.path.join(home, "hosts")))
            if e != "misterio.toml"
        ]
    else:
        misterio_host_list = misterio_host
    print(f"HOSTS:{misterio_host_list} MISTERIO HOME:{home}")
    if list_flag:
        for mhost in misterio_host_list:
            try:
                print(f"Roles for {mhost}")
                host_dir: str = os.path.join(home, "hosts", mhost)
                for filename in sorted(os.listdir(host_dir)):
                    print(f"\t{filename}")
            except FileNotFoundError:
                print(f"No roles for {mhost}")
        sys.exit(0)
    ######
    empty_check=("".join(docker_command)).strip()
    if empty_check=="":
        raise MisterioError("? Empty Command")        
    ######
    for mhost in misterio_host_list:
        docker_host: str = f"ssh://{mhost}"
        # GG: Localhost not accepted anymore: use misterio.toml instead
        if mhost == "localhost":
            raise MisterioConfigError(
                "Use misterio.toml not localhost for special needs"
            )
        # GG: Ensure DOCKER_CONTEXT is not defined
        os.environ.pop("DOCKER_CONTEXT",None)
        print("hip")
        os.environ["DOCKER_HOST"] = docker_host
        hosts_path = os.path.join(home, "hosts", mhost)
        load_misterio_config(os.path.join(home, "hosts", "misterio.toml"), mhost)
        for filename in sorted(os.listdir(hosts_path)):
            if filename.endswith(".env"):
                if single_role is None:
                    process_role(
                        home, os.path.join(hosts_path, filename), docker_command
                    )
                elif single_role in filename:
                    process_role(
                        home, os.path.join(hosts_path, filename), docker_command
                    )
            else:
                print(f"WARN Ignored not-env file: {filename}")
