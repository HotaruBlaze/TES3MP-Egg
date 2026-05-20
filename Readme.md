# TES3MP-Egg

[TES3MP](https://tes3mp.com/) is a project adding multiplayer functionality to [OpenMW](https://openmw.org/).

- **TES3MP Github**: [TES3MP/openmw-tes3mp](https://github.com/TES3MP/openmw-tes3mp)
- **Docker Image**: [hotarublaze/tes3mp-egg:0.8.1-debian12](https://hub.docker.com/repository/docker/hotarublaze/tes3mp-egg/tags/0.8.1-debian12)

---

## Features

- **TES3MP 0.8.1** - Full multiplayer support for OpenMW
- **Automatic Log Pruning** - Clean up old logs automatically
- **Custom Startup Commands** - Run custom commands on server start
- **Public IP Detection** - Help diagnose NAT issues
- **DreamWeave LuaJIT Support** - Optional performance-optimized LuaJIT build

---

## Server Ports

| Port | Protocol | Description |
|------|----------|-------------|
| 25565 | UDP | Game server port |

---

## Environment Variables

### Core Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_AUTOPRUNE` | `false` | Auto-delete logs older than 7 days from `.config/openmw` |
| `SHUTDOWN_TIMEOUT` | `30` | Graceful shutdown timeout in seconds (range: 5-300) |
| `TES3MP_PATH` | `/mnt/server` | Path to the TES3MP server directory |
| `CHECK_PUBLIC_IP` | `false` | Fetch and display public IP on startup to help diagnose NAT issues |

### Performance Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `USE_DREAMWEAVE_LUAJIT` | `false` | Enable DreamWeave-MP LuaJIT via LD_PRELOAD for improved performance |

---

## DreamWeave LuaJIT

When `USE_DREAMWEAVE_LUAJIT` is set to `true`, the server will automatically:

1. Download the latest **Stable-CI** build from [DreamWeave-MP/luajit2](https://github.com/DreamWeave-MP/luajit2)
2. Extract the `bin/libluajit.so` shared library to `/tmp/container/`
3. Set `LD_PRELOAD=/tmp/container/bin/libluajit.so` for all server processes

### Benefits

- **Performance Optimizations**: DreamWeave-MP's fork includes optimizations specifically for TES3MP
- **Better Compatibility**: Resolves issues with certain Lua scripts and mods
- **Automatic**: No manual installation required - just enable the toggle

### Source

The LuaJIT build is downloaded from:
```
https://github.com/DreamWeave-MP/luajit2/releases/download/Stable-CI/LuaJIT-Linux.7z
```

---

## Quick Start

1. **Install the Egg** in your Pterodactyl panel
2. **Configure Environment Variables** as needed
3. **Enable DreamWeave LuaJIT** (optional) for better performance
4. **Start the Server**

---

## Docker Build

The image is built from `debian:trixie-slim` with the following packages:
- curl
- ca-certificates
- libgl1
- libluajit-5.1-2
- libssl3t64
- p7zip-full (for LuaJIT extraction)

---

## Troubleshooting

### LuaJIT Loading Issues

If you see errors like:
```
ERROR: ld.so: object '/tmp/container/bin/libluajit.so' from LD_PRELOAD cannot be preloaded
```

Ensure:
1. The file exists at `/tmp/container/bin/libluajit.so`
2. The file has execute permissions (`chmod +x`)
3. The architecture matches your system (x86_64)

### Performance Issues

Try enabling `USE_DREAMWEAVE_LUAJIT=true` for improved Lua performance.

---

## Links

- [TES3MP Website](https://tes3mp.com/)
- [OpenMW](https://openmw.org/)
- [DreamWeave-MP LuaJIT](https://github.com/DreamWeave-MP/luajit2)
- [Pterodactyl Panel](https://pterodactyl.io/)