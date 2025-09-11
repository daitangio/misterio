"""
This module contains a set of utility to simplify and improve misterio management.
It is not required to use misterio, but you will like it
"""

import os, shutil, re, random
from datetime import datetime
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
@click.argument("source_host", nargs=1, type=str)
@click.argument("role_list", nargs=-1, type=str)
def misterio_rm(home, source_host, role_list):
    """
    Correctly remove a role from one host, ensuring proper cleanup is done.
    Move the configuration to the special attic directory
    """
    for role in role_list:
        misterio_cmd(
            home=home,
            list_flag=None,
            misterio_host=[source_host],
            single_role=role,
            docker_command=["down"],
        )
        src = os.path.join(home, "hosts", source_host, f"{role}.env")
        print(f"Moving {src} to the attic")
        attic_dir = os.path.join(home, "attic", source_host)
        os.makedirs(attic_dir, exist_ok=True)
        dst = os.path.join(attic_dir, f"{role}.env")
        shutil.move(src, dst)
        misterio_cmd(
            home=home,
            list_flag=None,
            misterio_host=[source_host],
            single_role=role,
            docker_command=["rebuild"],
        )


def write_prop(key, value, f):
    if " " in value:
        prop = f'{key.upper()}="{value}"'
    else:
        prop = f"{key.upper()}={value}"
    f.write(prop)
    f.write("\n")
    print(f"Defining:: {prop}")


def determine_instance_name(role):
    if "@" not in role:
        return role.lower()
    else:
        match = re.match(r"(.*)@(.*)", role)
        instance_name = match.group(2)
        return (match.group(1) + "_" + instance_name).lower()

def determine_fixed_port(role, base_port=7000):
    if "@" in role:
        match = re.match(r"(.*)@(.*)", role)
        instance_name = match.group(2)
        idx=int(instance_name)
    else:
        idx=0
    computed_port = idx+100*random.randint(1,100)
    return str(computed_port + base_port + idx)

@click.option(
    "--home",
    envvar="MISTERIO_HOME",
    default=os.getenv("PWD", ""),
    help="Home of hosts and roles folders. Can be set with MISTERIO_HOME",
)
@click.command("misterio_add")
@click.argument("target_host", nargs=1, type=str)
@click.argument("role_list", nargs=-1, type=str)
def misterio_add(home, target_host, role_list):
    """Add a role and pull the required images on the target host
    role can contain @inst to create multiple isstances like pippo@inst2
    For example

    misterio-add xwing pgvector@1 pgvector@2 

    To simplify initialization, misterio-add will add some default variables starting with MISTERIO_
    You can use them to parametrize your stack easily

    """
    for role in role_list:
        target_dir = os.path.join(home, "hosts", target_host)
        os.makedirs(target_dir, exist_ok=True)
        empty_file = os.path.join(target_dir, f"{role}.env")
        if os.path.exists(empty_file):
            print(f"FATAL: Role {role} already exists as {empty_file}")
            return
        # init a set of properties
        with open(empty_file, "w") as f:
            write_prop("MISTERIO_CREATION_USER", os.getenv("USER", "unknown"), f)
            write_prop(
                "MISTERIO_CREATION_DATE",
                datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
                f,
            )
            # See https://stackoverflow.com/questions/44924082/set-project-name-in-docker-compose-file
            write_prop("COMPOSE_PROJECT_NAME", determine_instance_name(role), f)
            write_prop("MISTERIO_MAGIPORT", determine_fixed_port(role), f)
        misterio_cmd(
            home=home,
            list_flag=None,
            misterio_host=[target_host],
            single_role=role,
            docker_command=["up", "-d"],
        )
