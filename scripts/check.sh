#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
project_dir=$(pwd)
mode=${1:-all}
if [ "$#" -gt 1 ]; then
  printf '%s\n' 'Usage: sh scripts/check.sh [rust|lean|all]' >&2
  exit 2
fi
case "$mode" in
  rust|lean|all) ;;
  *) printf '%s\n' 'Usage: sh scripts/check.sh [rust|lean|all]' >&2; exit 2 ;;
esac

# Generated evidence belongs with build output, not beside the source files.
log_dir="$project_dir/target/validation"
mkdir -p "$log_dir"
run() {
  check_name=$1
  shift
  printf 'Checking %s...\n' "$check_name"
  if ! "$@" > "$log_dir/$check_name.log" 2>&1; then
    cat "$log_dir/$check_name.log" >&2
    return 1
  fi
}

if [ "$mode" != lean ]; then
  run rust-format cargo fmt --all -- --check
  run rust-tests cargo test --locked
  run rust-no-std cargo test --locked --no-default-features
  run rust-release-tests cargo test --locked --release
  run rust-build cargo build --locked --release --lib
  run rust-clippy cargo clippy --locked --all-targets --all-features -- -D warnings
  run rust-docs env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
  run rust-example cargo run --locked --example roundtrip
fi

if [ "$mode" != rust ]; then
  cd "$project_dir/lean"
  run lean-build lake build
  run lean-axioms lake env lean Audit.lean
  if grep -En 'sorryAx|Lean\.ofReduceBool|error:' "$log_dir/lean-axioms.log"; then
    printf '%s\n' 'FAIL: unexpected proof dependency or audit error' >&2
    exit 1
  fi
  run lean-tests .lake/build/bin/relay_tests
  cat "$log_dir/lean-tests.log"
fi
printf 'PASS: %s checks. Logs: target/validation/\n' "$mode"
