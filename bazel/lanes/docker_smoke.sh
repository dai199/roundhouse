#!/bin/sh
# ci.yml smoke-campfire-docker: the archive's Dockerfile built and served.
# POSIX sh: the docker:dind image this runs in has no bash.
set -eu
tar xzf "$1"
cd campfire-docker
docker build -t campfire .
docker run -d --name campfire -p 3000:3000 campfire
i=0; while [ $i -lt 30 ]; do wget -q -O /dev/null http://localhost:3000/first_run && break; i=$((i+1)); sleep 1; done
code() { wget -S -O /dev/null "http://localhost:3000$1" 2>&1 | awk '/HTTP\//{c=$2} END{print c}'; }
root=$(code /); first=$(code /first_run); logo=$(code /account/logo)
echo "GET / -> $root; GET /first_run -> $first; GET /account/logo -> $logo"
docker rm -f campfire >/dev/null
[ "$root" = 302 ] && [ "$first" = 200 ] && [ "$logo" = 200 ]
