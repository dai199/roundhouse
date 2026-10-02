#!/bin/bash
# Runs the emitted Kotlin project's own tests, as tests/kotlin_toolchain.rs does.
set -euo pipefail
work="${TEST_TMPDIR:-$(mktemp -d)}/project"
mkdir -p "$work"
tar -xf "$1" -C "$work"
cd "$work"
gradle test --console=plain --no-daemon -q
