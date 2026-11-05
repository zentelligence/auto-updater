# Rule Index

Always read `AGENTS.md` first. Then route the task through this index.

| When the task touches | Read |
| --- | --- |
| Any production behaviour | `testing.md`, `../workflows/tdd.md`, `../workflows/change-loop.md` |
| Crate boundaries, traits, or the write pipeline | `architecture.md` |
| The manifest, `release.json`, `state.json`, or CLI command contracts | `release-contract.md` |
| Paths, archives, signatures, locking, or secrets | `security.md` |
| Rust code, dependencies, errors, or logging | `rust-quality.md` |
| Completion or CI | `../workflows/quality-gate.md` |
| Session memory, diaries, or the vault knowledge store | `memory.md` |
| Code review | `../reviewers/code-review.md` |
| Specification conformance review | `../reviewers/specification-review.md` |

For an implementation task, identify the governing specification section heading and fill a small change brief using `../templates/change-brief.md`. For a feature spanning several sections, maintain a matrix based on `../templates/requirement-test-matrix.md`.

Source-of-truth precedence:

1. Explicit user instruction.
2. The closest applicable `AGENTS.md`.
3. The specification.
4. The decisions log.
5. `.claude` guidance.
6. Existing code and tests, which may reveal drift but do not silently override the written contract.

When two authoritative sources conflict, stop the affected design decision, show the exact conflict, and ask for resolution. Continue with independent work when safe.

## Language

- Australian English spelling, grammar, and punctuation.
- Never use em dashes or double-hyphen substitutes.
