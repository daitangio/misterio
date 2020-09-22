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

if [ ! -z $MISTERIO_HOST_ALIAS  ]; then
    MISTERIO_HOST=$MISTERIO_HOST_ALIAS
else 
    MISTERIO_HOST=$HOSTNAME    
fi

set -e
#set -v
mkdir -p hosts/$MISTERIO_HOST/states
cd $(dirname $0)
for role_env in hosts/$MISTERIO_HOST/*.env ; do
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

# Status generation: store it in the machine
#  docker ps  -a --format "{{json .}}"

(
    cd hosts/$MISTERIO_HOST/states ; 
    docker ps  --format "table {{.ID}},{{.Names}}" >new-state.csv
    if [  -f current-state.csv ] ; then
     diff current-state.csv new-state.csv  || true 
    fi 
)
(cd hosts/$MISTERIO_HOST/states ;  cp new-state.csv current-state.csv )
done
