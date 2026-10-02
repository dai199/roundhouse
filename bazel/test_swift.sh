#!/bin/bash
# Runs the emitted Swift package's own tests, as tests/swift_toolchain.rs does.
set -euo pipefail
work="${TEST_TMPDIR:-$(mktemp -d)}/project"
mkdir -p "$work"
tar -xf "$1" -C "$work"
cd "$work"
# Not in the swift image: the CSQLite system library the package links.
if [ ! -f /usr/include/sqlite3.h ] && command -v apt-get >/dev/null; then
  apt-get update -qq && apt-get install -y -qq libsqlite3-dev >/dev/null
fi
# Not SwiftPM's own sandbox: on macOS it cannot nest inside Bazel's.
swift build --disable-sandbox
swift test --disable-sandbox
