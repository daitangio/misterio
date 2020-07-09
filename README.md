# misterio
Docker-compose based Ansible alternative. It hates spiders

# So what?
Misterio is a set of two tiny bash script you can use to "apply" a set of roles to a infinte numbers of hosts.

The only dependency is a recent version of docker-compose.

It do not relay on docker swarm: every host can be tiny and isolated.

# How
For every hostname, define a directory inside hosts/
Put it a env file based on this syntax:
    <rolename>[@inst].env
where @inst is OPTIONAL and can be used to have multiple instance on the same machine.
You need to parametrize the role to support this.

# The magic
For every roles on the target machine misterio will:
1. copy the corret env file.
2. pass the command you provide to docker-compose
3. fail fast or loop

The "apply" pseudo-command will do a build and up in one step


# Distributed 
A misterio-ssh demo script is provided to show how to propagate it on a set of remote hosts
Misterio ssh need a misterio command followed by a list of target:

```bash
./misterio-ssh.sh apply wonderhost
./misterio-ssh.sh logs wonderhost
./misterio-ssh.sh down wonderhost
```

