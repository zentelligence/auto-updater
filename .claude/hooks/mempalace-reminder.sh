#!/usr/bin/env bash
# Stop hook: remind Claude to record findings/lessons/progress in mempalace before ending the session.
# stop_hook_active guards against blocking forever when there is genuinely nothing to record.
set -uo pipefail

payload="$(cat)"
stop_hook_active="$(jq -r '.stop_hook_active // false' <<<"${payload}")"
transcript_path="$(jq -r '.transcript_path // empty' <<<"${payload}")"

[[ "${stop_hook_active}" == "true" ]] && exit 0
[[ -n "${transcript_path}" && -f "${transcript_path}" ]] || exit 0

if grep -q 'mcp__mempalace__' "${transcript_path}"; then
  exit 0
fi

printf 'No mempalace tool call this session. Findings, lessons, and progress are recorded in mempalace (agent "claude-updater"), not left only in chat or repo files. Record this session'"'"'s outcome (mempalace_diary_write, mempalace_kg_add, etc.) before stopping, or explain why there is nothing worth recording.\n' >&2
exit 2
