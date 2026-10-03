#!/bin/bash
# The Rails oracle campfire's lanes compare against, prepared once and cached on campfire's pin and the scripts.
# Its gems are bundled inside (vendor/bundle), as the image's gem home is not an output.
# Usage: campfire_oracle.sh CAMPFIRE_TGZ PACK_PY OUT_TAR
set -euo pipefail
root=$PWD; work=$(mktemp -d)
mkdir -p "$work/campfire" "$work/repo" "$work/home"
export HOME="$work/home" PATH="/usr/local/bundle/bin:$PATH"
tar -xzf "$1" -C "$work/campfire" --strip-components=1
cp -RL scripts "$work/repo/"
redis-server --daemonize yes --save "" --appendonly no >/dev/null
# Not left running: a daemon still holding the action's output keeps it from ending.
trap 'redis-cli shutdown nosave >/dev/null 2>&1 || true' EXIT
(cd "$work/repo" && BUNDLE_PATH=vendor/bundle scripts/campfire-oracle prepare --app "$work/campfire" >/dev/null)
python3 "$root/$2" "$work/repo/build/campfire-oracle" "$3"
