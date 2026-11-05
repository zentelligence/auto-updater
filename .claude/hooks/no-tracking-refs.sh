#!/usr/bin/env bash
# PostToolUse hook: report tracking-identifier references (R-12, D-7, Q-3, VD-9, "phase 2") leaking
# into docs, comments, or code immediately after an edit.
# AGENTS.md: never point at a decision or task identifier that exists only in chat history or the
# vault decisions log; explain the WHY in terms a reader can verify against the code or the
# specification instead.
# Exit 2 feeds the message back to Claude without blocking the completed edit.
set -uo pipefail

payload="$(cat)"
file_path="$(jq -r '.tool_input.file_path // empty' <<<"${payload}")"

case "${file_path}" in
  *.rs|*.md) ;;
  *) exit 0 ;;
esac

[[ -f "${file_path}" ]] || exit 0

matches="$(grep -nE '\b[A-Z]+-[0-9]+\b|\bphase[[:space:]]*[0-9]+\b' -i "${file_path}" || true)"

if [[ -n "${matches}" ]]; then
  printf 'tracking-identifier reference in %s: remove the R-*/D-*/Q-*/VD-*/"phase N" reference and explain the WHY in terms verifiable from the code or specification instead.\n%s\n' \
    "${file_path}" "${matches}" >&2
  exit 2
fi

exit 0
