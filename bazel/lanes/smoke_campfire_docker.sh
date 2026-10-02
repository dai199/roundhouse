#!/bin/bash
# ci.yml build-campfire-archive + smoke-campfire-docker: the archive's Dockerfile built and served.
set -euo pipefail
scripts/build-campfire-archive --out _site "$CAMPFIRE_APP"
tar xzf _site/campfire/docker.tgz
cd campfire-docker
docker build -t campfire .
docker run -d --name campfire -p 3000:3000 campfire
for i in $(seq 1 30); do curl -sf -o /dev/null localhost:3000/first_run && break; sleep 1; done
root=$(curl -s -o /dev/null -w '%{http_code}' localhost:3000/)
first=$(curl -s -o /dev/null -w '%{http_code}' localhost:3000/first_run)
logo=$(curl -s -o /dev/null -w '%{http_code}' localhost:3000/account/logo)
echo "GET / -> $root; GET /first_run -> $first; GET /account/logo -> $logo"
docker rm -f campfire >/dev/null
[ "$root" = 302 ] && [ "$first" = 200 ] && [ "$logo" = 200 ]
