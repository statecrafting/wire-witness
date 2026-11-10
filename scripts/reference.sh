#!/bin/sh
# Spec: specs/020-readme-and-reference-documentation/spec.md
#
# Generates or checks docs/reference, one spec-spine reference page per spec,
# with the repository-local engine. Generation needs a fresh ledger, and the
# pages are hashed inputs of the index, so `generate` compiles and indexes,
# writes the pages, then indexes again. `check` never writes and exits
# non-zero when a page is missing, changed, orphaned or malformed, or the
# ledger is stale.
set -eu
cd "$(dirname "$0")/.."
engine=.bin/spec-spine
request=docs/reference/request.json
case "${1:-}" in
  generate)
    "$engine" compile >/dev/null
    "$engine" index >/dev/null
    "$engine" reference generate --request "$request"
    "$engine" index >/dev/null
    ;;
  check) exec "$engine" reference check --request "$request" ;;
  *) echo "usage: scripts/reference.sh generate|check" >&2; exit 3 ;;
esac
