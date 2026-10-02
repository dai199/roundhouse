#!/bin/bash
# Runs the emitted Swift package's own tests, as tests/swift_toolchain.rs does.
set -euo pipefail
work="${TEST_TMPDIR:-$(mktemp -d)}/project"
mkdir -p "$work"
tar -xf "$1" -C "$work"
cd "$work"
# Not SwiftPM's own sandbox: on macOS it cannot nest inside Bazel's.
swift build --disable-sandbox
swift test --disable-sandbox
