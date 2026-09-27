#!/usr/bin/env bash
# Spec: specs/001-boundaries-and-authority/spec.md
# Enforces the repository's authored-content rules over files or supplied text.

set -uo pipefail

SELF="scripts/check-authored-content.sh"
status=0
mode=tree

if [ "${1:-}" = "--text" ]; then
  mode=text
  shift
  [ "$#" -gt 0 ] || {
    echo "check-authored-content: --text needs at least one file" >&2
    exit 3
  }
  SELF=""
else
  cd "$(git rev-parse --show-toplevel 2>/dev/null)" || {
    echo "check-authored-content: not inside a git work tree" >&2
    exit 3
  }
fi

if [ "${BASH_VERSINFO[0]:-0}" -lt 3 ]; then
  echo "check-authored-content: needs bash 3.2 or newer" >&2
  exit 3
fi

files=()
if [ "$mode" = text ]; then
  for file in "$@"; do
    [ -f "$file" ] || {
      echo "check-authored-content: no such file: $file" >&2
      exit 3
    }
    files+=("$file")
  done
else
  while IFS= read -r -d '' file; do
    case "$file" in
      .statecraft/derived/*|*.png|*.jpg|*.jpeg|*.gif|*.ico|*.pdf) continue ;;
    esac
    [ -f "$file" ] && files+=("$file")
  done < <(git ls-files -z --cached --others --exclude-standard)
fi

[ "${#files[@]}" -gt 0 ] || {
  echo "check-authored-content: no authored files found" >&2
  exit 3
}

emdash=$(printf '\xe2\x80\x94')
em_hits=$(grep -n -F -- "$emdash" "${files[@]}" 2>/dev/null)
if [ -n "$em_hits" ]; then
  echo "U+2014 is refused by the authored-content rules:"
  printf '%s\n' "$em_hits" | sed 's/^/  /'
  status=1
fi

scan=()
for file in "${files[@]}"; do
  [ "$file" = "$SELF" ] || scan+=("$file")
done

patterns='codex\.ai/code/session_[0-9A-Za-z_-]+
Codex-Session:
Co-[Aa]uthored-[Bb]y:.*(Claude|Codex|Copilot|Gemini)
Generated with.*(Claude|Codex)'

if [ "${#scan[@]}" -gt 0 ]; then
  link_hits=$(printf '%s\n' "$patterns" | while IFS= read -r pattern; do
    grep -n -E -- "$pattern" "${scan[@]}" 2>/dev/null
  done)
  if [ -n "$link_hits" ]; then
    echo "agent attribution or session tracking is refused:"
    printf '%s\n' "$link_hits" | sed 's/^/  /'
    status=1
  fi
fi

if [ "$status" -eq 0 ]; then
  echo "check-authored-content: ${#files[@]} authored file(s) clean"
fi
exit "$status"
