# Misterio: so what?
Docker-compose based Ansible/SaltStack/NameYour *minimalistic alternative*.
<img align="right"   src="https://gioorgi.com/wp-content/uploads/2020/07/misterio-300x170.png" alt="Mysterio Marvel" >
It is super-easy to use.

Misterio is a set of two tiny bash script you can use to "apply" a set of roles to a infinite numbers of hosts.
Less then 100 lines of bash code (sorry Ansible :)

Misterio is able to manage a set of compose target as an one, appling status changes easily.


# Why?
1. The only dependency is a recent version of `docker`.
2. It does not rely on docker swarm or on K8s. It can run even on ultra-small nano containers on Amazon, provided you have little swap (tested)
3. It is agent-less. It depends only on `docker` and `bash` on the target.
4. Everything must be versioned to work: you cannot easily "forget" something on your local machine.


# How
For every hostname, define a directory inside `hosts/`
Put in it an `env` file based on this syntax:

    <rolename>[@inst].env

where `@inst` is OPTIONAL and can be used to have multiple instances on the same machine.


# The magic
For every role on the target machine misterio will:
1. copy the correct `env` file.
2. pass the command you provide to `docker-compose`
3. fail fast or loop

The "apply" pseudo-command will do a `build` and `up` in one step

*NEW!* You can use the pseudo command --list to get the list of all the roles, and the --<rolename> syntax to apply command only to a role.


# Distributed 
A misterio-ssh demo script is provided to show how to propagate it on a set of remote hosts.
Misterio ssh needs a `misterio` command followed by a list of targets:

```bash
./misterio-ssh apply pi@raspy1 peter@mayhome parker@newserver
./misterio-ssh logs pi@raspy1 peter@mayhome parker@newserver
./misterio-ssh down pi@raspy1 peter@mayhome parker@newserver
```

Misterio-ssh is quite smart; for every target it will
1. Clone an ultra-small version of the repository and send it over the wire to the selected target
   `misterio-ssh` will try to use `rsync` and fallback to `scp` if needed
2. Remote launch it
3. Stop if an error occurs before step (1)
   Proceed to the next target if it fails

# The Bonus
Misterio is also a collection of ready-made docker-compose infrastructure you can jump into.
For instance, jenkins-with-docker show you how to get a dockerized-jenkins with:

- self running git server
- access to docker daemon to self-build stuff using docker plugin


# Tips
Under docker for Windows, add
COMPOSE_CONVERT_WINDOWS_PATHS=1
to your env path if you plan to bind stuff like
> /var/run/docker.sock:/var/run/docker.sock

This will enable your roles to run on Windows and on Linux dameons seamlessly.
See https://stackoverflow.com/a/52866439/75540 for more details

# The Hype
1. It is trivial to parallelize `misterio-ssh` or the replace `docker compose` with K8s clusters (try and push me back).
2. You can add git submodules below `roles/` to link recipes (your personal "ansible galaxy" is docker hub!)
3. No complex stuff to learn: it is just DOCKER!

# Other alternative
https://github.com/piku/piku is an heroku-like alternative, based on python and not requiring docker.

