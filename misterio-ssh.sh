#!/bin/bash
set -e 
cd $(dirname $0)
misterio_cmd=$1
shift 1

for target in $* ; do 
    # Clone repo.
    # Send to target
    # run misterio
    # cleanup
    MISTERIO_HOME_DIR=/tmp/misterio-pack-$target
    if [ -d $MISTERIO_HOME_DIR  ]; then
        echo REMOVE $MISTERIO_HOME_DIR
        echo before proceeding
        exit 1
    fi
    set -u -x
    git clone --depth 1 -b master file://$(pwd) $MISTERIO_HOME_DIR
    rsync -z  --delete $MISTERIO_HOME_DIR $target:$MISTERIO_HOME_DIR || scp -r $MISTERIO_HOME_DIR $target:$MISTERIO_HOME_DIR 
    ssh $target $MISTERIO_HOME_DIR/misterio.sh $misterio_cmd || {
        echo Misterio FAILED on $target
        echo Removing temp stuff
        ssh $target rm -r $MISTERIO_HOME_DIR    
    }

    ssh $target rm -r $MISTERIO_HOME_DIR
    rm -rf $MISTERIO_HOME_DIR
    set +x
done