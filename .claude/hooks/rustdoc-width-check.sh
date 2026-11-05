#!/usr/bin/env bash
# PostToolUse hook: report Rustdoc comment lines over 120 characters immediately after a Rust file
# edit. AGENTS.md: Rustdoc wraps at 120 characters, not the rustfmt/rustdoc default of 80.
# Exit 2 feeds the message back to Claude without blocking the completed edit.
set -uo pipefail

payload="$(cat)"
file_path="$(jq -r '.tool_input.file_path // empty' <<<"${payload}")"

case "${file_path}" in
  *.rs) ;;
  *) exit 0 ;;
esac

[[ -f "${file_path}" ]] || exit 0

over_limit="$(awk '/^[[:space:]]*(\/\/\/|\/\/!)/ && length($0) > 120 { printf "%d: %s\n", NR, $0 }' "${file_path}")"

if [[ -n "${over_limit}" ]]; then
  printf 'Rustdoc line(s) over 120 characters in %s (wrap at 120, not 80):\n%s\n' \
    "${file_path}" "${over_limit}" >&2
  exit 2
fi

exit 0
