# Release Contract Rules

This project's public compatibility boundary is not a network API; it is the on-disk and wire formats other things depend on: `release.json`, the signed update manifest, `state.json`, the installation directory layout, the active-pointer format on each platform, and the `updater-cli` subcommand surface (`update check`, `update`, `update --yes`, `rollback`, `update cleanup`, `update diagnostics`, `install --version`). Treat every one of these as a versioned contract, per the specification's "Release Metadata", "Signed Update Manifest", "Local State", and "Update Commands" sections.

## Implementation

- Deserialize `release.json`, the update manifest, and `state.json` into request/response DTOs that reject unknown or malformed fields as the format requires, then convert to validated domain types with `From` or `TryFrom`. Never use ad hoc parsing functions at this boundary.
- Bump `schema_version` (or the equivalent field) deliberately when a format changes in a way an older reader cannot ignore; document what changed and why in the commit, and note it in the decisions log if it changes this project's own on-disk contract rather than just its Rust representation.
- CLI output intended for scripting (`update check`, `update diagnostics`) must be structured or otherwise parseable on a documented, stable basis; do not make a script parse prose that was only ever written for a human.
- Preserve stable, matchable error variants and an actionable human message. A validation failure names which check failed (see the specification's "Bundle Validation" list) rather than a generic "invalid release".
- Every public command declared in `release.json` must have a corresponding stable launcher installed, and vice versa; a mismatch is a rejected release set, not a warning.

## Contract tests

For each on-disk or wire format, cover:

1. the smallest valid document and a representative complete one;
2. every documented validation failure from the specification's "Bundle Validation" and "Failure Handling") as its own named test;
3. unknown fields, missing required fields, and schema-version mismatches;
4. Windows-specific fields (`.exe` suffixes, `current.txt`/`previous.txt`) alongside the Unix-like equivalents (`current`/`previous` symlinks), never asserted as identical by assumption; and
5. the CLI subcommand's exit code and structured output for both success and each documented failure.

Test these through the crate that owns the format (`updater-core` for `release.json`/manifest/state parsing, `updater-cli` for subcommand behaviour), not only through an end-to-end script.
