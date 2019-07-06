#!/bin/bash
sleep 5

scriptLog="[TES3MP-EGG]"
logDirectory="/home/container/.config/openmw"

if [ "$LOG_AUTOPRUNE" = "true" ]; then
    if [ -d "$logDirectory" ]; then
        echo "$scriptLog AutoPruning logs older than 7 days..."
        find "$logDirectory" -type f -name '*.log' -mtime +7 -exec rm {} \;
    fi
    if [ ! -d "$logDirectory" ]; then
        echo "$scriptLog AutoPruning enabled but $logDirectory does not exist, skipping"
    fi
fi

cd /home/container || exit

if test -f ./tes3mp-prelaunch; then bash ./tes3mp-prelaunch "$WRAPPER"; fi
LD_LIBRARY_PATH="./lib" exec ./tes3mp-server.x86_64 