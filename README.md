# misterio
Docker-compose based Ansible alternative. It hates spiders and it is super-easy to use.

# So what?
Misterio is a set of two tiny bash script you can use to "apply" a set of roles to a infinte numbers of hosts.

# Why ?
1. The only dependency is a recent version of docker-compose.
2. It do not relay on docker swarm or on K8s. It can run even on ultra-small nano containers.
3. It is agent-less. It depends only on docker compose and bash
4. Everything must be verionated to work: you cannot "forget" something.


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
./misterio-ssh.sh apply pi@raspy1 peter@mayhome parker@newserver
./misterio-ssh.sh logs pi@raspy1 peter@mayhome parker@newserver
./misterio-ssh.sh down pi@raspy1 peter@mayhome parker@newserver
```

