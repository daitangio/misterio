import os, sys, shutil, subprocess
import click


def is_rebuild(cmdlist):
    if len(cmdlist) == 1:
        if cmdlist[0] == "rebuild":
            return True
    return False


def process_role(home, env_full_path, docker_command):
    if is_rebuild(docker_command):
        # rebuild require two command to run:
        low_level_pr(home, env_full_path, ["down"])
        low_level_pr(home, env_full_path, ["up", "--build", "-d"])
    else:
        low_level_pr(home, env_full_path, docker_command)


def low_level_pr(home, env_full_path, docker_command):
    env_file = os.path.basename(env_full_path)
    if "@" in env_file:
        role_name = env_file.split("@")[0]
    else:
        role_name = env_file.split(".env")[0]
    dirz = os.path.join(home, "roles", role_name)
    full_command = ["docker", "compose"]
    full_command.extend(docker_command)
    docker_host = os.environ["DOCKER_HOST"]
    print(f"==== {role_name} \t-> {full_command}")
    os.chdir(dirz)
    shutil.copyfile(env_full_path, ".env")
    try:
        subprocess.run(full_command, check=True)
    except subprocess.CalledProcessError as e:
        print(f"{docker_host}::{role_name} Failed with return code {e.returncode}")

def verify_misterio_home(home:str):
    error_count=0
    for d in ["hosts", "roles"]:
        pathz=os.path.join(home,d)
        if not os.path.isdir(pathz):
            print(f"FATAL: Missed required directory {pathz}")
            error_count+=1
    if error_count > 0:
        raise Exception(f"home dir has {error_count} validation errors")


@click.command("misterio")
@click.option(
    "--home",
    envvar="MISTERIO_HOME",
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
@click.version_option(version="0.1.2")
@click.argument("docker_command", nargs=-1, type=str)
def misterio(home, list_flag, misterio_host, single_role, docker_command):
    """M I S T E R I O
    docker compose-based alternative to K8s/Ansible/SaltStack
    By default the system will scan all the hostname inside
    $MISTERIO_HOME/hosts/
    and connect to every of them using DOCKER_HOST=ssh://<hostname> for connection

    Verify logs of all services to just one server:

        misterio -h wonderboy -- logs --tail 5

    Verify clustered elastic-service on all nodes:

        misterio --single-role elastic-service ps

    Verify elastic service on just two nodes, named wonderboy and adam:

        misterio -h wonderboy -h adam --single-role elastic-service ps

    // Special Internal Commands //

    * Rebuild the entire system to ensure everything is configured properly:

        mistero rebuild

    """
    return misterio_cmd(home, list_flag, misterio_host, single_role, docker_command)


def misterio_cmd(home, list_flag, misterio_host, single_role, docker_command):
    verify_misterio_home(home)
    if misterio_host is None or len(misterio_host) == 0:
        misterio_host_list = os.listdir(os.path.join(home, "hosts"))
    else:
        misterio_host_list = misterio_host
    print(f"MISTERIO HOME:{home} Host to be processed:{misterio_host_list}")
    if list_flag:
        for mhost in misterio_host_list:
            try:
                print(f"Roles for {mhost}")
                for filename in os.listdir(os.path.join(home, "hosts", mhost)):
                    print(f"\t{filename}")
            except FileNotFoundError:
                print(f"No roles for {misterio_host}")
        sys.exit(0)
    for mhost in misterio_host_list:
        docker_host = f"ssh://{mhost}"
        os.environ["DOCKER_HOST"] = docker_host
        print(f"=== {docker_host} ===")
        hosts_path = os.path.join(home, "hosts", mhost)
        for filename in os.listdir(hosts_path):
            # print(filename)
            if filename.endswith(".env"):
                if single_role is None:
                    process_role(
                        home, os.path.join(hosts_path, filename), docker_command
                    )
                else:
                    if single_role in filename:
                        process_role(
                            home, os.path.join(hosts_path, filename), docker_command
                        )
