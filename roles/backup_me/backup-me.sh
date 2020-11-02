#!/bin/sh

echo Backup start.
df -h /host_to_backup
ls /host_to_backup/
set -e -x -u 
echo rsync -avuzb --exclude '*~' /host_to_backup/ ${RSYNC_BACKUP_SERVER}:${RSYNC_BACKUP_SERVER_PATH}
#rsync -Cavuzb . samba:samba/
echo Backup done

