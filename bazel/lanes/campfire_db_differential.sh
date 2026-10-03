#!/bin/bash
# ci.yml campfire-compare's db-differential step, and campfire-db-differential-spinel with --spinel.
# Usage: campfire_db_differential.sh ORACLE_TAR [EMIT_TAR] [-- OPTIONS...]
set -euo pipefail
oracle=build/campfire-oracle; mkdir -p "$oracle"; tar -xf "$1" -C "$oracle"; shift
# Not under the oracle: the image's gem home is where its `bin/rails` looks.
cp -a "$oracle"/vendor/bundle/ruby/*/. /usr/local/bundle/
reuse=()
if [[ $# -gt 0 && "$1" != -- ]]; then mkdir -p "$TEST_TMPDIR/emit"; tar -xf "$1" -C "$TEST_TMPDIR/emit"; reuse=(--reuse "$TEST_TMPDIR/emit"); shift; fi
[[ "${1:-}" == -- ]] && shift
redis-server --daemonize yes --save "" --appendonly no >/dev/null
# Not left running: a daemon still holding the test's output keeps the test from ending.
trap 'redis-cli shutdown nosave >/dev/null 2>&1 || true' EXIT
scripts/campfire-db-differential "${reuse[@]}" "$@" "$CAMPFIRE_APP"
