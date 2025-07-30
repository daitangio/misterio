from requests import *
import click
import os,sys,shutil,subprocess

def process_role(home, env_full_path, docker_command):
    env_file=os.path.basename(env_full_path)
    if len(docker_command) == 0 or docker_command==["apply"]:        
        docker_command=["up", "--build", "-d"]
    if "@" in env_file:
        role_name=env_file.split("@")[0]
    else:
        role_name=env_file.split(".env")[0]    
    dirz=os.path.join(home,'roles',role_name)
    # DEBUG print(f"{role_name} {docker_command} {dirz} {env_full_path}")
    full_command=["docker", "compose"]
    full_command.extend(docker_command)
    print(f"{role_name} -> {full_command}")
    os.chdir(dirz)    
    shutil.copyfile(env_full_path,".env")
    subprocess.run(full_command)


@click.command('misterio')
@click.option('--home', envvar='MISTERIO_HOME', 
              default=os.getenv("PWD",""),
              help="Home of hosts and roles folders. Required to work")
@click.option('--misterio-host', envvar=["HOSTNAME", "MISTERIO_HOST"], default=os.getenv("HOSTNAME","") )
@click.option('--list/--no-list', help="List things", default=False)
@click.option('--single-role', envvar='MISTERIO_SINGLE_ROLE', default=None)
@click.version_option(version="1.0.0")
@click.argument('docker_command', nargs=-1, type=str)
def misterio(home, list, misterio_host, single_role, docker_command):
    print(f"HOME:{home} {list} {misterio_host}")
    if list:
        print(f"Roles for {misterio_host}")
        for filename in os.listdir(os.path.join(home, 'hosts', misterio_host)):
            print(filename)
        sys.exit(0)
    if single_role:
        print(f"Processing single role {single_role}")
        # opt="$1"
        # shift 1
        # single_role=${opt//--/}
        # # echo $single_role
        # processRole hosts/$MISTERIO_HOST/${single_role}.env $*
    else:
        print("Role processing")
        hosts_path=os.path.join(home, 'hosts', misterio_host)
        for filename in os.listdir(hosts_path):
            #print(filename)
            if filename.endswith(".env"):
                process_role(home, os.path.join(hosts_path,filename), docker_command)
        # cd $(dirname $0)
        # for role_env in hosts/$MISTERIO_HOST/*.env ; do
        #     processRole $role_env $*
        # done
        pass

if __name__ == '__main__':
    misterio()