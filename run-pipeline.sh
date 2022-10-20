#!/bin/bash
# Misterio pipeline builder
# https://devopscube.com/run-docker-in-docker/
set -euo pipefail
#  ./run-pipeline.sh pipelines/backup_demo/
# Can be useful to have the same docker process for multiple builds...
# pivoting on docker-compose
pipeline_dir=$(realpath $1)
builder=$(basename $pipeline_dir)_builder
echo Pipeline context: $pipeline_dir Container: $builder
backupImage() {
    archive_space="./.pipeline_cache"
    mkdir -p $archive_space    
    set -x
    docker exec -ti -w /root/  $builder docker save -o image.tar pipline:latest
    docker exec -ti $builder docker images
    docker cp ${builder}:/root/image.tar ${archive_space}/${builder}.image.tar
    set +x
}

runBuilder(){
    set -x
    docker run --privileged -d \
        -v $pipeline_dir:/root/ \
        --name $builder docker:dind
}

# MAIN
# Run the service
echo Staring pipeline $builder
runBuilder || {
        echo "Starting cached container"
        docker start $builder 
}
set +x
# waitDaemon
while true; do
    docker logs --tail=5 $builder 2>&1 | grep "API listen" && { 
        break;
    }
    sleep 0.5
done

echo "Preloaded cached images (if any)"
docker exec -ti $builder docker images
docker exec $builder  sh -c 'ls -l /root/' # Dockerfile ; df -h'
docker exec -w /root/  $builder  docker build -t pipline:latest -f ./Dockerfile .

# Optional backupImage
echo -n Stopping pipeline
docker stop $builder
