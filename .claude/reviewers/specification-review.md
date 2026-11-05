# Specification Conformance Lens

Use this review for behaviour, on-disk/wire format, or scope-decision claims.

1. List the affected specification section headings and any relevant decisions log entries in scope.
2. Compare fields, defaults, limits, results, and error conditions against the relevant section (`Release Metadata`, `Signed Update Manifest`, `Local State`, `Update Commands`, and similar).
3. Check the write pipeline, lock/failure policy, and platform parity (Unix symlink pointer versus Windows `current.txt`/`previous.txt`) where applicable.
4. Map each acceptance statement in the specification's "Acceptance Criteria" and each row of its "Failure Handling" table to a named test.
5. Identify unspecified behaviour that became a de facto public contract (an undocumented CLI flag, an implicit field default) without being recorded anywhere.

Classify gaps as:

- **violation:** contradicts the specification or a recorded decision;
- **missing:** required behaviour or test is absent;
- **drift:** code, docs, formats, or tests disagree;
- **ambiguity:** the specification and the decisions log do not decide the behaviour; or
- **future scope:** valid idea outside the current change.

Never resolve an ambiguity by silently choosing the easiest implementation. Present the alternatives and their compatibility consequences, and note that the ambiguity needs a decisions log entry once resolved.
