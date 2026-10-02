#!/bin/bash
# ci.yml campfire-db-differential-spinel.
set -euo pipefail
redis-server --daemonize yes --save "" --appendonly no >/dev/null
# Not left running: a daemon still holding the test's output keeps the test from ending.
trap 'redis-cli shutdown nosave >/dev/null 2>&1 || true' EXIT
scripts/campfire-oracle prepare --app "$CAMPFIRE_APP"
scripts/campfire-db-differential --spinel "$CAMPFIRE_APP"
