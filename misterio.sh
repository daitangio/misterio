#!/bin/bash

if [ "z$1" == "z" ]; then
cat <<EOF
Usage: misterio.sh \{ apply \| up \| start \|  down \}
Every docker compose command is supported

apply: make a build and an up

Mistery pick every role listed inside hosts/$HOSTNAME and process it
the role env file format is
    <rolename>[@inst].env
where @inst can be used to have multiple instance on the same machine

The technology is based only on docker compose (no docker swarm)


EOF
exit 1
fi

set -e
# set -v
cd $(dirname $0)
for role_env in hosts/$HOSTNAME/*.env ; do
    role=$(basename $role_env)
    inst_re='(.*)@(.*).env'
    if [[ $role  =~ $inst_re ]]; then        
        MISTERIO_INST_NAME=${BASH_REMATCH[2]}
        #echo MISTERIO_INST_PREFIX=$MISTERIO_INST_PREFIX
        role_dir=roles/${BASH_REMATCH[1]}
    else
        unset MISTERIO_INST_NAME
        role_dir=roles/${role//.env/}
    fi

    echo Applying $role into ${role_dir}

    set -x
    cp ${role_env}  ${role_dir}/.env    
    { set +x; } 2>/dev/null

    if [ "$#" == "1" -a "$1" == "apply" ] ; then    
        ( cd $role_dir ; 
            docker-compose up --build -d || (echo FAILED $role on $HOSTNAME ))
    else
        ( cd $role_dir ; docker-compose $* || (echo FAILED $role on $HOSTNAME ))
    fi
    { set +x; } 2>/dev/null
    # docker build 'https://github.com/elastic/dockerfiles.git#v6.8.10:elasticsearch'
done
echo '=========================='
echo -ne 'Total Memory used:'
docker stats --no-stream --format "table {{.MemPerc}}" | sed 's/[A-Za-z]*//g' | awk '{sum += $1} END {print sum "%"}'
docker ps

