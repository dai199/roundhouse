#!/bin/bash
# Runs the emitted Rust project's own tests, as tests/rust_toolchain.rs does.
set -euo pipefail
# Not on the test PATH: the rust image installs cargo under /usr/local/cargo, and Bazel does not take the image's ENV.
if [ -x /usr/local/cargo/bin/cargo ]; then
  export PATH="/usr/local/cargo/bin:$PATH" CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup
fi
tarball="$1"
work="${TEST_TMPDIR:-$(mktemp -d)}/project"
mkdir -p "$work"
tar -xf "$tarball" -C "$work"
cd "$work"
cargo test --quiet
