#!/bin/bash
# --run_under for every test: when the fixtures arrive as symlinks (local execution's runfiles), the
# apps' test/ trees are rebuilt as real files, since ingest deliberately skips symlinked test files.
# On a remote executor the inputs are real files and this only execs.
set -euo pipefail
if [[ -d fixtures && -n "$(find fixtures -maxdepth 3 -path '*/test/*' -type l -print -quit 2>/dev/null)" ]]; then
  tree="${TEST_TMPDIR:-$(mktemp -d)}/real-test-dirs"; mkdir -p "$tree/fixtures"
  for e in * .[!.]*; do [[ -e "$e" && "$e" != fixtures ]] && ln -s "$PWD/$e" "$tree/$e"; done
  for app in fixtures/*; do
    if [[ -d "$app/test" ]]; then
      mkdir -p "$tree/$app"
      for e in "$app"/* "$app"/.[!.]*; do [[ -e "$e" && "${e##*/}" != test ]] && ln -s "$PWD/$e" "$tree/$e"; done
      # Hard links, not copies: real files to ingest, at the cost of a directory walk.
      cp -RLl "$app/test" "$tree/$app/test" 2>/dev/null || cp -RL "$app/test" "$tree/$app/test"
    else
      ln -s "$PWD/$app" "$tree/$app"
    fi
  done
  cd "$tree"
fi
exec "$@"
