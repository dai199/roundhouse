#!/bin/bash
# The emit campfire's suite runs against, prepared as scripts/campfire-suite prepares it, and the
# strict spinel emit's error count: what lane_campfire_conformance checks, cached on the emit's bytes.
# Usage: campfire_suite_emit.sh ROUNDHOUSE CAMPFIRE_TGZ PACK_PY OUT_TAR OUT_ERRORS
set -euo pipefail
root=$PWD; work=$(mktemp -d)
mkdir -p "$work/bins" "$work/campfire" "$work/home"
cp "$root/$1" "$work/bins/roundhouse"
tar -xzf "$2" -C "$work/campfire" --strip-components=1
export ROUNDHOUSE_BINS="$work/bins" PATH="$root/bazel/shim:/usr/local/bundle/bin:$PATH" HOME="$work/home"
"$root/scripts/campfire-suite" --emit-only --out "$work/emit" "$work/campfire" >/dev/null
python3 "$3" "$work/emit" "$4"
{ "$work/bins/roundhouse" --target spinel "$work/campfire" -o "$work/strict" 2>&1 || true; } | grep -c 'error\[' > "$5" || true
