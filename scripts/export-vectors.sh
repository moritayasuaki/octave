#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
project_dir=$(pwd)
mkdir -p target/validation tests/fixtures
(
  cd lean
  lake build relay_vectors
) > target/validation/lean-vectors.log 2>&1 || {
  cat target/validation/lean-vectors.log >&2
  exit 1
}
temporary=$(mktemp "$project_dir/tests/fixtures/lean-vectors.txt.XXXXXX")
trap 'rm -f "$temporary"' 0
trap 'exit 1' HUP INT TERM
lean/.lake/build/bin/relay_vectors > "$temporary"
chmod 644 "$temporary"
mv "$temporary" tests/fixtures/lean-vectors.txt
printf '%s\n' 'Exported public fixtures from the Lean core.'
