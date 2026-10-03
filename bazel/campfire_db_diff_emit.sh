#!/bin/bash
# campfire-db-differential's transpile (campfire + the scenario overlay) alone, so its run is cached on the emit's bytes.
# Usage: campfire_db_diff_emit.sh ROUNDHOUSE CAMPFIRE_TGZ PACK_PY OUT_TAR
set -euo pipefail
root=$PWD; work=$(mktemp -d)
mkdir -p "$work/bins" "$work/campfire" "$work/home"
cp "$root/$1" "$work/bins/roundhouse"
tar -xzf "$2" -C "$work/campfire" --strip-components=1
export ROUNDHOUSE_BINS="$work/bins" PATH="$root/bazel/shim:/usr/local/bundle/bin:$PATH" HOME="$work/home"
"$root/scripts/campfire-db-differential" --emit-only "$work/emit" "$work/campfire" >/dev/null
python3 "$3" "$work/emit" "$4"
