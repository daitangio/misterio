#!/bin/bash
set -v 
# Recive pack is NEEDED to be able to push and modifications
su jenkins -c 'git daemon  --detach  --reuseaddr  --verbose  --export-all --enable=receive-pack --enable=upload-archive --base-path=/agos'
# Run the salt minion
# BUG MUST RUN AS ROOT
salt-minion -d
su jenkins -c '/usr/local/bin/jenkins.sh'