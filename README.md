# What is Misterio?
Docker-compose based Ansible/SaltStack/NameYour *minimalistic alternative*.
<img align="right"   src="https://gioorgi.com/wp-content/uploads/2020/07/misterio-300x170.png" alt="Mysterio Marvel" >
It is super-easy to use.

*Cool!* The new python version is easier to use and understand.

Misterio is a python command you can use to "apply" a set of roles to a infinite numbers of hosts.
Less then 150 lines of python code HELP INCLUDED (sorry Ansible :)

Misterio is able to manage a set of compose target as an one, appling status changes easily.

## Simple usage example

Suppose to have two hosts called alice and bob. You want to run elasticsearch on both of them, and one gitlab instance on bob.
So you define:

```sh
misterio_project/               # Misterio home directory
├── hosts/
│   ├── alice/
│   │   └── elasticsearch.env  # empty file 
│   └── bob/
│       └── gitlab.env         # empty file 
|           elasticsearch.env  # empty file
└── roles/
    ├── elasticsearch/
    │   └── docker-compose.yml
    └── gitlab/
        └── docker-compose.yml 
```

Then running something like

    misterio --home ./misterio_project rebuild

will build the service and run them.
To see the log you can use

    misterio --home ./misterio_project -- logs --tail 10

For simple stats on a single host:

    misterio -h xwing -- stats --no-stream

You can further customize the roles, adding variable inside the elasticsearch.env file (like Elastic Search cluster details)

# Why?
1. The only dependency is a recent version of `docker` CE  (on target hosts) and `python` 3 (on misterio host). 
2. It does not rely on docker swarm or on K8s. It can run even on ultra-small nano containers on Amazon (1GB RAM), provided you have a little swap (tested)
3. It is agent-less. It depends only on `docker daemon` on the target. Docker communication is done via ssh and can be further configured via the `.ssh/config` file (for instance to setup keys, tunneling, etc)
4. Everything must be versioned to work: you cannot easily "forget" something on your local machine. It respect the Infrastructure as Code paradigm. 

# Details on env file
For every hostname, define a directory inside `hosts/`
Put in it an `env` file based on this syntax:

    <rolename>[@inst].env

where `@inst` is OPTIONAL and can be used to have multiple instances of a role on the same machine. Misterio will configure them one by one.


# The magic
For every role on the target machine misterio will:
1. for each role, copy the correct `env` file calling it .env
2. pass the command you provide to `docker-compose`
3. fail fast or loop

The "rebuild" pseudo-command will do a `down` + `build` and `up` in one step.



# Distributed 
Because misterio manage the DOCKER_HOST automatically, it is already distributed.

# Python official version
Look at https://pypi.org/project/misterio/ for the latest version

# Python development version
Install on your virtualenv with

```sh
    python -m venv .venv
    . .venv/bin/activate
    pip install -e .
    misterio --help
```

# Support command

*misterio-mv* command can be used to migrate a staless service from one host to another.
It remove (compose down) the source service, move the env file and then reboot (up -d) the other one.

*misterio-rm* command delete a role from a host, ensuring it is destroied and no dandling instances are kept.


# The Bonus: stacks
Misterio is also a collection of ready-made docker-compose infrastructure you can jump into.
For instance, jenkins-with-docker show you how to get a dockerized-jenkins with:

- self running git server
- access to docker daemon to self-build stuff using docker plugin


# Tips

You can use the pseudo command --list to get the list of all the roles, and the --single-role option to restrict only to a role.

Under docker for Windows, add
COMPOSE_CONVERT_WINDOWS_PATHS=1
to your env path if you plan to bind stuff like
> /var/run/docker.sock:/var/run/docker.sock

This will enable your roles to run on Windows and on Linux dameons seamlessly.
See https://stackoverflow.com/a/52866439/75540 for more details


# The Hype
1. You can add git submodules below `roles/` to link recipes (your personal "ansible galaxy" is... docker hub!)
2. No complex stuff to learn: it is just DOCKER!

# Podman

Podman is not tested, and it could require a modification to the way the DOCKER_HOST variable is addressed too. anyway, if you are able to create a pull request with a --podman option, I will be happy to merge it.



# Other alternative
https://github.com/piku/piku is an heroku-like alternative, based on python and not requiring docker.

# Legacy
The old misterio bash version can be found under [./old_sh_version](./old_sh_version) folder: it is a 4 years old version, which can still be used if want to further reduce depencencies on misterio controlling host.


