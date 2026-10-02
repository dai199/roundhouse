#!/bin/bash
# ci.yml build-campfire-archive + smoke-campfire: the archive's README run against itself.
set -euo pipefail
scripts/build-campfire-archive --out _site "$CAMPFIRE_APP"
(cd e2e/campfire && npm ci --no-audit --no-fund)
scripts/smoke --tgz _site/campfire/spinel.tgz campfire
