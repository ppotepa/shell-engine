#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Build C# sidecar upfront so there's no compile delay when the game loads it
echo "[build] Building cognitOS sidecar..."
dotnet build -c Release mods/shell-quest/os/cognitOS/cognitOS.csproj --nologo -v q

exec ./launcher.sh --mod shell-quest --audio "$@"
