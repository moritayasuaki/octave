#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p .local/validation

# Lake fixes this cache entry point. Keep actual storage in the local workspace.
if [ -L lean/.lake ]; then
  if [ "$(readlink lean/.lake)" != "../.local/lean" ]; then
    printf '%s\n' 'Unexpected lean/.lake link; inspect it before moving the cache.' >&2
    exit 1
  fi
elif [ -e lean/.lake ]; then
  if [ -e .local/lean ] || [ -L .local/lean ]; then
    printf '%s\n' 'Both lean/.lake and .local/lean exist; resolve the cache conflict first.' >&2
    exit 1
  fi
  mv lean/.lake .local/lean
fi
mkdir -p .local/lean
if [ ! -L lean/.lake ]; then
  ln -s ../.local/lean lean/.lake
fi
