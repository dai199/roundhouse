#!/bin/bash
# The DOM compare lane for the Rust target, as `scripts/compare rust` runs it:
# Rails and the emitted Rust server side by side, diffed by roundhouse-compare.
set -euo pipefail
fixture_tar="$PWD/$1"; project_tar="$PWD/$2"; compare_bin="$PWD/$3"
work="${TEST_TMPDIR:-$(mktemp -d)}"
# Not on the test PATH: an image's toolchains under their own prefixes.
for d in /usr/local/cargo/bin "$HOME/.cargo/bin" /usr/local/bundle/bin; do [ -d "$d" ] && PATH="$d:$PATH"; done
export PATH
if ! command -v cargo >/dev/null; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain 1.98.1
  PATH="$HOME/.cargo/bin:$PATH"
fi
mkdir -p "$work/real-blog" "$work/project"
tar -xf "$fixture_tar" -C "$work/real-blog"
tar -xf "$project_tar" -C "$work/project"
mkdir -p "$work/real-blog/tmp/pids" "$work/real-blog/log"
(cd "$work/real-blog" && bundle install --quiet)
mkdir -p "$work/project/storage"
cp "$work/real-blog/storage/development.sqlite3" "$work/project/storage/development.sqlite3"
(cd "$work/project" && cargo build --release --quiet)

pids=()
trap 'for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done' EXIT
(cd "$work/real-blog" && exec bin/rails server -p 4000 -e development >"$work/rails.log" 2>&1) & pids+=($!)
(cd "$work/project" && PORT=3000 exec ./target/release/app >"$work/app.log" 2>&1) & pids+=($!)
for url in http://localhost:4000/articles http://localhost:3000/articles; do
  for _ in $(seq 1 120); do curl -sf -o /dev/null "$url" && break; sleep 1; done
  curl -sf -o /dev/null "$url" || { echo "not up: $url"; grep -m3 -B2 -A3 -E "Error|Exception" "$work/rails.log"; tail -5 "$work/app.log"; exit 1; }
done
args=(--reference http://localhost:4000 --target http://localhost:3000)
for p in / /articles /articles/1 /articles/new /articles/1/edit /articles.json /articles/1.json; do args+=(--path "$p"); done
"$compare_bin" "${args[@]}"
