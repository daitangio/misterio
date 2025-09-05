"""
This module contains a set of utility to simplify and improve misterio management.
It is not required to use misterio, but you will like it
"""

import os, shutil
import click
from .misterio import misterio_cmd


@click.option(
    "--home",
    envvar="MISTERIO_HOME",
    default=os.getenv("PWD", ""),
    help="Home of hosts and roles folders. Can be set with MISTERIO_HOME",
)
@click.command("misterio_mv")
@click.argument("role", nargs=1, type=str)
@click.argument("source_host", nargs=1, type=str)
@click.argument("destination_host", nargs=1, type=str)
def misterio_mv(home, role, source_host, destination_host):
    """Move a service from a host to another"""
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[source_host],
        single_role=role,
        docker_command=["down"],
    )
    src = os.path.join(home, "hosts", source_host, f"{role}.env")
    dst = os.path.join(home, "hosts", destination_host, f"{role}.env")
    print(f"{src} -> {dst}")
    shutil.move(src, dst)
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[destination_host],
        single_role=role,
        docker_command=["up", "-d"],
    )


@click.option(
    "--home",
    envvar="MISTERIO_HOME",
    default=os.getenv("PWD", ""),
    help="Home of hosts and roles folders. Can be set with MISTERIO_HOME",
)
@click.command("misterio_mv")
@click.argument("role", nargs=1, type=str)
@click.argument("source_host", nargs=1, type=str)
def misterio_rm(home, role, source_host):
    """
    Correctly remove a role from one host, ensuring proper cleanup is done.

    """
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[source_host],
        single_role=role,
        docker_command=["down"],
    )
    src = os.path.join(home, "hosts", source_host, f"{role}.env")
    print(f"Removing {src}")
    # os.unlink(src)
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[source_host],
        single_role=role,
        docker_command=["rebuild"],
    )
