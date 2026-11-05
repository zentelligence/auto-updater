# Testing Strategy

Tests prove externally meaningful behaviour and the invariants the specification states as requirements. A large test count is not a substitute for covering the specified contracts.

## Test layers

| Layer | Purpose | Preferred dependencies |
| --- | --- | --- |
| Unit | Pure validation, parsing, version comparison, and error branches | In-memory values and focused fakes |
| Property | Path-confinement and version-ordering invariants over broad generated inputs | `proptest`-style generators with reproducible seeds |
| Integration | Real filesystem, atomic replacement, archive extraction, and a local HTTP fixture | Isolated temporary directories and a fake/local server, never the live network |
| Release-contract | `release.json`, manifest, and `state.json` schema and CLI subcommand behaviour | In-process crate composition with deterministic services (see `release-contract.md`) |
| Platform | POSIX symlink activation and Windows locked-executable/pointer-file semantics, asserted separately | The matching CI runner; mark a case that can only run on one OS as such rather than skipping it silently |
| Acceptance | End-to-end install, upgrade, rollback, and cleanup drills against a disposable installation root | A throwaway directory standing in for the real installation root |

## Required patterns

- Use one temporary installation root per test. Never read or mutate the operator's real `~/.local/share`, `%LOCALAPPDATA%`, `~/.local/bin`, or `PATH`.
- Make fixture intent visible in its name and keep fixtures minimal. Put larger conformance corpora under `fixtures/` with provenance and expected results.
- Assert both state and side effects: file bytes, the active/previous pointer, `state.json` contents, and which files were left untouched, not just a returned success value.
- For failures, assert the specific error variant and that no prohibited state changed (the active pointer, `previous`, and `state.json` in particular).
- Test crash/interruption boundaries with controllable fault injection around flush and rename steps, not flaky process timing.
- Keep manual acceptance scripts repeatable and document the exact host OS and product version used.

## Coverage priorities

Every documented failure in the specification's "Failure Handling" table must be reachable by a named test. High-risk matrices take priority over line coverage: path forms and platforms, fresh install versus upgrade versus rollback, a locked old executable during activation, and concurrent update attempts.

Never use snapshots as the sole assertion for a security decision (signature verification, checksum verification, path confinement). Review snapshot changes semantically, and keep stable contract fields asserted directly.
