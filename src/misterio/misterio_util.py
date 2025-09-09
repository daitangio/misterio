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
@click.command("misterio_rm")
@click.argument("role", nargs=1, type=str)
@click.argument("source_host", nargs=1, type=str)
def misterio_rm(home, role, source_host):
    """
    Correctly remove a role from one host, ensuring proper cleanup is done.
    Move the configuration to the special attic directory
    """
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[source_host],
        single_role=role,
        docker_command=["down"],
    )
    src = os.path.join(home, "hosts", source_host, f"{role}.env")
    print(f"Moving {src} to the attic")
    attic_dir=os.path.join(home,"attic",source_host)
    os.makedirs(attic_dir,exist_ok=True )
    dst=os.path.join(attic_dir, f"{role}.env")
    shutil.move(src, dst)
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[source_host],
        single_role=role,
        docker_command=["rebuild"],
    )

@click.option(
    "--home",
    envvar="MISTERIO_HOME",
    default=os.getenv("PWD", ""),
    help="Home of hosts and roles folders. Can be set with MISTERIO_HOME",
)
@click.command("misterio_add")
@click.argument("role", nargs=1, type=str)
@click.argument("target_host", nargs=1, type=str)
def misterio_add(home, role, target_host):
    """ Add a role and pull the required images on the target host
    """
    target_dir=os.path.join(home, "hosts", target_host)
    os.makedirs(target_dir,exist_ok=True )
    empty_file = os.path.join(target_dir, f"{role}.env")
    if os.path.exists(empty_file):
        print(f"FATAL: Role {role} already exists as {empty_file}")
        return
    # Create an empty file (overwrite if exists)
    with open(empty_file, "w") as f:
        pass
    misterio_cmd(
        home=home,
        list_flag=None,
        misterio_host=[target_host],
        single_role=role,
        docker_command=["pull"],
    )
