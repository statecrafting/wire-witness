#!/usr/bin/env bash
# Spec: specs/001-boundaries-and-authority/spec.md
# Verifies the initial three-crate graph before product behavior is added.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT

package_set() {
  cargo tree --locked --edges normal,build --depth 1 --prefix none -p "$1" \
    | sed -E 's/ v[0-9][^ ]*.*$//; s/ \(\*\)$//' \
    | sort -u
}

assert_set() {
  package=$1
  expected=$2
  actual="$scratch/$package.actual"
  package_set "$package" > "$actual"
  if ! diff -u "$expected" "$actual"; then
    echo "check-boundaries: unexpected dependency set for $package" >&2
    exit 1
  fi
}

printf '%s\n' 'wire-witness-core' > "$scratch/core.expected"
printf '%s\n' 'wire-witness-core' 'wire-witness-proxy' > "$scratch/proxy.expected"
printf '%s\n' 'wire-witness-cli' 'wire-witness-core' 'wire-witness-proxy' > "$scratch/cli.expected"

assert_set wire-witness-core "$scratch/core.expected"
assert_set wire-witness-proxy "$scratch/proxy.expected"
assert_set wire-witness-cli "$scratch/cli.expected"

echo "check-boundaries: exact three-crate dependency graph"
