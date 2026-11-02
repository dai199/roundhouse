#!/usr/bin/env bash
# Runs one spinel-framework shard: TESTS is its suites.
set -uo pipefail
# The binaries run outside cargo, so .cargo/config.toml's [env] stack size is not applied.
export RUST_MIN_STACK="${RUST_MIN_STACK:-33554432}"
test -n "${TESTS:-}"
tmp="${RUNNER_TEMP:-/tmp}/spinel-framework"
mkdir -p "$tmp/logs"

args=()
for suite in $TESTS spinel_bundle; do args+=(--test "$suite"); done
cargo test --no-run --message-format=json "${args[@]}" > "$tmp/build.json" || exit 1
exe() {
  python3 - "$tmp/build.json" "$1" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["name"] == sys.argv[2] and "test" in m["target"]["kind"]:
        print(m["executable"])
PY
}
# param_binds* also hold JRuby tests, run in compare-jruby.
filter() {
  case "$1" in param_binds|param_binds_values|param_binds_cleanup) echo _spinel ;; esac
}
run_suite() {
  local suite=$1
  local bin
  bin=$(exe "$suite")
  python3 scripts/ci-plan.py timed --kind framework --name "$suite" -- \
    "$bin" --ignored --nocapture $(filter "$suite") > "$tmp/logs/$suite.log" 2>&1
}
export -f exe filter run_suite
export tmp

# Not a compile per contract: the suites' run_spinel contracts are collected
# (each test stops at its contract) and built in shared groups first.
start=$(date +%s)
for suite in $TESTS; do
  echo "$suite"
done | RH_SPINEL_BUNDLE_COLLECT="$tmp/bundle-cases" xargs -P "$(( $(nproc) * 2 ))" -I{} \
  bash -c 'timeout 90 "$(exe {})" --ignored $(filter {}) >/dev/null 2>&1 || true'
echo "collected $(ls "$tmp/bundle-cases" 2>/dev/null | wc -l) contracts in $(( $(date +%s) - start ))s"
RH_SPINEL_BUNDLE_COLLECT="$tmp/bundle-cases" RH_SPINEL_BUNDLE_OUT="$tmp/spinel-bundles" \
  "$(exe spinel_bundle)" --ignored --exact spinel_bundle_build --nocapture || true
echo "bundled in $(( $(date +%s) - start ))s"
export RH_SPINEL_BUNDLE="$tmp/spinel-bundles"

# Not one suite at a time: most now run in seconds; the PostgreSQL one shares a database, so it runs alone.
status=0
parallel=()
serial=()
for suite in $TESTS; do
  case "$suite" in spinel_pg_db) serial+=("$suite") ;; *) parallel+=("$suite") ;; esac
done
printf '%s\n' "${parallel[@]}" | xargs -P 3 -I{} bash -c 'run_suite {}' || status=1
for suite in "${serial[@]}"; do run_suite "$suite" || status=1; done
for suite in $TESTS; do
  echo "::group::$suite"
  cat "$tmp/logs/$suite.log"
  echo "::endgroup::"
  grep -qE "^test result: FAILED|error: test failed|panicked at" "$tmp/logs/$suite.log" && echo "::error::$suite failed"
done
echo "finished in $(( $(date +%s) - start ))s"
exit "$status"
