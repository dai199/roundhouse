#!/usr/bin/env bash
# Experiment: put a `spinel` ahead of spinel-dist on PATH that adds --incremental to binary builds.
set -euo pipefail
real="$GITHUB_WORKSPACE/spinel-dist/spinel"
if [ "${SPINEL_INC:-0}" != 1 ]; then echo "spinel --incremental off"; exit 0; fi
dir="$RUNNER_TEMP/spinel-inc"
mkdir -p "$dir" "$RUNNER_TEMP/spinel-cache"
cat > "$dir/spinel" <<W
#!/bin/sh
for a in "\$@"; do
  case "\$a" in -c|-S|-v|--version|--print-build|--emit-*|--ext|--ext-*|--incremental) exec "$real" "\$@";; esac
done
exec "$real" --incremental "\$@"
W
chmod +x "$dir/spinel"
echo "$dir" >> "$GITHUB_PATH"
echo "SPINEL_CACHE_DIR=$RUNNER_TEMP/spinel-cache" >> "$GITHUB_ENV"
echo "spinel --incremental on"
