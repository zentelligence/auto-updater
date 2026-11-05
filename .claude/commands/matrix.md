---
description: Create or update the requirement-to-test matrix for the current work
argument-hint: <feature or specification section>
---

Maintain the requirement-to-test matrix for: $ARGUMENTS

1. Read `.claude/templates/requirement-test-matrix.md` for the required structure, and an existing matrix under `docs/development/` for the established level of detail, if one exists.
2. Keep rows at observable-contract granularity: specification section heading, contract summary, test layer, and named tests once paths exist.
3. Maintain the error-coverage table against the specification's "Failure Handling" rows, and the gate-evidence table against its "Acceptance Criteria" bullets.
4. Set status honestly: `green` only when the named test exists and passed in a run you performed; `red` for an observed failing test; `pending` otherwise. Note platform-specific items (Windows locking, macOS/Linux acceptance) as pending with the reason.
5. Save under `docs/development/`, following the existing naming if a matrix already exists there.
