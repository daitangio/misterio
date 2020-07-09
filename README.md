# Misterio
Docker-compose based Ansible alternative. It hates spiders and it is super-easy to use.

# So what?
Misterio is a set of two tiny bash script you can use to "apply" a set of roles to a infinte numbers of hosts.

# Why ?
1. The only dependency is a recent version of docker-compose.
2. It do not relay on docker swarm or on K8s. It can run even on ultra-small nano containers on amazon, provided you have little swap (tested)
3. It is agent-less. It depends only on docker compose and bash
4. Everything must be versionated to work: you cannot "forget" something.
5. you can set misterio public repo as upstream, merge to your code and kept your secret hosts in your private repository (hosts dir is ignored by design in git: you must force its versioning).


# How
For every hostname, define a directory inside hosts/
Put it a env file based on this syntax:
    <rolename>[@inst].env
where @inst is OPTIONAL and can be used to have multiple instance on the same machine.
You need to parametrize the role to support this, because we avoid using "magic way" to pass the inst value.

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

Misterio-ssh is quite smart; for every target it will
1. Clone a ultra-small version of the repository and send it over the wire to the selected target
   mistrio-ssh will try to use rsync and fallback to scp if needed
2. Remote launch it
3. Stop if an error occurs before step (1)
   Proceed to the next target if it fails



# The Hype
1. It is trivial to parallelize misterio-ssh or the replace docker-compose with K8s clusters (try and push me back).
2. You can add git submodules below roles/ to link recipe (your personal "ansible galaxy" is docker hub!)

