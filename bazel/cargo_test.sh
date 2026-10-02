#!/bin/bash
# Runs the emitted Rust project's own tests, as tests/rust_toolchain.rs does.
set -euo pipefail
tarball="$1"
work="${TEST_TMPDIR:-$(mktemp -d)}/project"
mkdir -p "$work"
tar -xf "$tarball" -C "$work"
cd "$work"
cargo test --quiet
