#!/bin/bash

scriptLog="[TES3MP-EGG]"
logDirectory="/home/container/.config/openmw"

# Handle custom LuaJIT if enabled
if [ "$USE_DREAMWEAVE_LUAJIT" = "true" ]; then
    echo "$scriptLog Dreamweave LuaJIT enabled, downloading..."
    mkdir -p /tmp/container
    curl -sL -o /tmp/container/LuaJIT-Linux.7z "https://github.com/DreamWeave-MP/luajit2/releases/download/Stable-CI/LuaJIT-Linux.7z"
    7z x /tmp/container/LuaJIT-Linux.7z -o/tmp/container -y > /dev/null 2>&1
    chmod +x /tmp/container/bin/libluajit.so
    export LD_PRELOAD="/tmp/container/bin/libluajit.so"
    echo "$scriptLog Using Dreamweave Luajit loaded via LD_PRELOAD: $LD_PRELOAD"
fi

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
    exec env LD_PRELOAD="$LD_PRELOAD" bash -c "$STARTUP"
fi

# Default behavior for tes3mp-runner
if [ -f "/home/container/tes3mp-runner" ]; then
    exec env LD_PRELOAD="$LD_PRELOAD" /home/container/tes3mp-runner
fi

# Fallback for tes3mp-server
cd /home/container || exit
if test -f ./tes3mp-prelaunch; then bash ./tes3mp-prelaunch "$WRAPPER"; fi
exec env LD_PRELOAD="$LD_PRELOAD" LD_LIBRARY_PATH="./lib" ./tes3mp-server.x86_64