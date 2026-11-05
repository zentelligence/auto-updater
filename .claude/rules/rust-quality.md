# Rust Quality Rules

## Language and APIs

- Forbid unsafe code at crate or workspace level.
- Production paths contain no `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!`.
- Implement every conversion between project types as `From<T>` when it cannot fail or `TryFrom<T>` when it can. Use `Into` or `TryInto` at call sites when that improves inference or readability.
- Never add free-form conversion functions or inherent conversion methods such as `to_*`, `from_*`, `into_*`, `convert_*`, or `parse_*`. A method that performs domain behaviour rather than representation conversion must be named for that behaviour.
- Avoid semantic and narrowing conversions with `as`; use checked `TryFrom` conversions. Third-party codec primitives may be called inside the crate that owns that concern, but conversion into or out of project types still crosses a `From` or `TryFrom` implementation.
- Use one focused `thiserror` enum per crate. Avoid catch-all variants and opaque string errors at domain boundaries.
- Use newtypes for validated installation-root paths, release-relative paths, version identifiers, content hashes, and other security- or contract-significant values.
- Prefer exhaustive matches. If forward compatibility needs an unknown case (for example, an unrecognised `schema_version`), make its behaviour explicit and tested.
- Use `tracing` fields for diagnostics. Redact secrets and full local paths at the point structured fields are created.
- Keep public APIs documented and small. Default to crate visibility until a cross-crate consumer exists.

## Blocking work

- This is a synchronous CLI: no async runtime. Blocking filesystem, network, and archive I/O runs directly on the calling thread; there is no executor thread to keep it off.
- Make interruption safe: a process killed mid-update must not leave a partial write, a half-promoted version directory, or an inconsistent active pointer. The write pipeline in `architecture.md` exists specifically to make that true.
- Avoid wall-clock sleeps in tests. Drive timers and lock-wait behaviour through an injected clock or scheduler.
- Apply timeouts and size limits at network and archive-extraction boundaries.

## Dependencies

- Add a dependency only when the standard library or an existing dependency is insufficient and the abstraction belongs in this repository. Justify the choice in the commit that adds it.
- Disable unnecessary default features and document any native/system build needs.
- Keep dependencies out of `updater-core` unless they support pure domain modelling (`semver`, `serde`, and similar); network, archive, and signature-verification crates belong in `updater-engine`.
- Run `cargo audit`; review licences and maintenance posture before accepting a new dependency.

Formatting and lint cleanliness are requirements. Do not add broad `allow` attributes. A narrow allow must explain the invariant and be covered by a test where practical.

## Holistic quality

Mandate: holistic quality trumps everything else.

Review a change as part of the whole system. A locally elegant function is not high quality if it weakens a boundary, duplicates policy, obscures operations, hurts another platform, or makes the public contract harder to evolve. Do not trade correctness, security, maintainability, test strength, observability, or documentation for speed or a smaller diff.

- Unit tests never live inline. For every `foo.rs` with unit tests, create a sibling `foo_test.rs` and wire it up with:
      #[cfg(test)]
      #[path = "foo_test.rs"]
      mod tests;
  `foo_test.rs` starts with `use super::*;` to reach private items.
- Integration tests stay under `tests/`, one file per test target, unchanged from standard Rust convention.
- No source file (code or test) over 1000 lines or 50kB. When a file approaches the limit, split it by responsibility. Do not "solve" a near-limit code file by moving tests out, since they're already separate.
