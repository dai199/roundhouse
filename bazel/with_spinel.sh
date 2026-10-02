#!/bin/bash
# Runs a test binary with the Bazel-built spinel toolchain on PATH and --ignored,
# as ci.yml's framework-tests-spinel stages spinel-dist before `cargo test -- --ignored`.
# Usage: with_spinel.sh SPINEL_DIST_TAR TEST_BINARY [ARGS...]
set -euo pipefail
dist="${TEST_TMPDIR:-$(mktemp -d)}/spinel-dist"
mkdir -p "$dist"
tar -xf "$1" -C "$dist"
chmod +x "$dist/spinel" "$dist/spin" "$dist/spinel_rbs_extract"
export PATH="$dist:/usr/local/cargo/bin:$PATH"
bin="$2"; shift 2
exec "$bin" --ignored "$@"
