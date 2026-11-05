# Architecture Rules

The workspace is a one-way dependency graph, not a hexagon with a single composition root: there is no server process, so there is no adapter layer to keep implementation details out of. There is still a hard boundary between domain types/ports and the code that performs I/O against them.

## Dependency boundaries

- `updater-core` owns manifest and release metadata types, local state, validated/confined path types, error types, and the atomic-write and activation-port traits. It depends on nothing else in this workspace, and only on dependencies pulled in for pure domain modelling (see `rust-quality.md`).
- `updater-launcher` and `updater-engine` depend on `updater-core` only. Neither depends on the other, and neither depends on `updater-cli`.
- `updater-cli` is the composition point: it depends on `updater-core`, `updater-launcher`, and `updater-engine`, translates CLI arguments into domain calls, and translates results back into operator-facing output and exit codes.
- Keep OS- and transport-specific types out of domain function signatures. Convert at the boundary with `From` or `TryFrom` and validate before calling into `updater-core` types. Do not hide that translation in free-form helper functions.

## The write pipeline

Every mutation to the active pointer, `previous`, or `state.json` follows this order:

```text
request -> validate -> acquire update.lock -> temp write + flush + atomic rename/replace
        -> release update.lock
```

A completed activation is the primary contract. A release set is validated in full, including running every declared executable's version check, before any pointer changes; a single validation failure rejects the whole release set rather than activating part of it. A locked old executable on Windows defers cleanup; it never turns a successful activation into a reported failure (see the specification's "Cleanup Rules" and "Failure Handling" sections).

## Concurrency and testability

- This is a synchronous, sequential CLI: no async runtime. Serialise the whole update transaction behind `update.lock`; do not attempt finer-grained concurrency inside one invocation without a recorded decision.
- Inject the clock, the HTTP transport, the hash function, and the signature verifier wherever determinism or failure-path testing requires it. Prefer small capability traits over a generic service locator.
- Keep derived/staging data (`staging/`, `downloads/`) rebuildable and confined to the resolved installation root.

Any proposed boundary change must state which crate(s) it affects, the dependency direction, the failure policy, and how it will be tested before implementation, and should usually be recorded as a new entry in the decisions log.
