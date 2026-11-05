---
description: Draft a change brief for a piece of the specification before implementation
argument-hint: <specification section(s) or a feature description>
---

Draft a change brief for: $ARGUMENTS

1. Read `.claude/templates/change-brief.md` for the required structure, and `.claude/workflows/change-loop.md` for the discovery steps.
2. Identify the governing specification section heading(s) and any relevant decisions log entries. Read them in full. If the specification is unavailable or ambiguous for this scope, record the uncertainty and stop; do not invent semantics.
3. Inspect the working tree and the code and tests nearest the change so the brief reflects the actual baseline, not an assumed one.
4. Fill every section of the template: traceability, behaviour (given, when, then, must not), scope, holistic quality assessment, planned TDD evidence, verification commands, and risks.
5. Surface contract ambiguities explicitly per the source-of-truth precedence in `.claude/rules/00-index.md` rather than resolving them silently.
6. Present the brief in the conversation. Save it under `docs/development/` (matching a `<short-outcome>-change-brief.md` naming) only when the work is large enough to be worth keeping, or the user asks.

Stop after the brief. Do not begin implementation until the user confirms scope, or the task explicitly authorised implementation.
