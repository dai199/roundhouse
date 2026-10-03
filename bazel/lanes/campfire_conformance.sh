#!/bin/bash
# ci.yml campfire-conformance: the strict-emit ceiling, then campfire's own suite against the emit, held to the floor.
# Usage: campfire_conformance.sh SUITE_EMIT_TAR STRICT_ERRORS_FILE
set -euo pipefail
gem install sqlite3 bcrypt nokogiri webmock mocha rqrcode ruby-vips sentry-ruby platform_agent concurrent-ruby net-http-persistent web-push rails-html-sanitizer --no-document >/dev/null
ruby tests/campfire_suite_bcrypt.rb
errors=$(cat "$2")
echo "campfire strict emit (spinel, no --allow-unsupported): $errors errors (ceiling 0)"
[ "$errors" -le 0 ]
work="${TEST_TMPDIR:-$(mktemp -d)}"; mkdir -p "$work/emit"; tar -xf "$1" -C "$work/emit"
scripts/campfire-suite --reuse "$work/emit" --tally "$work/tally.txt" --fail-log "$work/failures.txt" --json "$work/summary.json"
tests=$(awk -F'|' '{p += $3} END {print p+0}' "$work/tally.txt")
files=$(grep -c '^PASS' "$work/tally.txt" || true)
echo "campfire conformance: $tests tests, $files files green (floor ${FLOOR_TESTS}/${FLOOR_FILES})"
[ "$tests" -ge "$FLOOR_TESTS" ] && [ "$files" -ge "$FLOOR_FILES" ]
