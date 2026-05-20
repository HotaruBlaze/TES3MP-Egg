#!/bin/bash

scriptLog="[TES3MP-EGG]"
logDirectory="/home/container/.config/openmw"

# Handle LOG_AUTOPRUNE
if [ "$LOG_AUTOPRUNE" = "true" ]; then
    if [ -d "$logDirectory" ]; then
        echo "$scriptLog AutoPruning logs older than 7 days..."
        find "$logDirectory" -type f -name '*.log' -mtime +7 -exec rm {} \;
    fi
    if [ ! -d "$logDirectory" ]; then
        echo "$scriptLog AutoPruning enabled but $logDirectory does not exist, skipping"
    fi
fi

# Handle STARTUP environment variable
if [ -n "$STARTUP" ]; then
    echo "$scriptLog Executing STARTUP command: $STARTUP"
    exec bash -c "$STARTUP"
fi

# Default behavior for tes3mp-runner
if [ -f "/home/container/tes3mp-runner" ]; then
    exec /home/container/tes3mp-runner
fi

# Fallback for tes3mp-server
cd /home/container || exit
if test -f ./tes3mp-prelaunch; then bash ./tes3mp-prelaunch "$WRAPPER"; fi
exec LD_LIBRARY_PATH="./lib" ./tes3mp-server.x86_64