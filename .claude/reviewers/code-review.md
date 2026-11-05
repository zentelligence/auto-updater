# Code Review Lens

Review the diff, not the author's intent. Report concrete findings ordered by severity, with file/line evidence and the violated invariant.

## Correctness and integrity

- Does every activation, rollback, and state write take the validated, lock-aware, atomic pipeline in `architecture.md`?
- Can an interrupted download, extraction, validation, promotion, or activation leave the active pointer, `previous`, or `state.json` inconsistent?
- Is a release set rejected as a whole on any single executable's validation failure, never partially activated?
- Is a locked old executable on Windows handled as deferred cleanup, never as a failed activation?
- Are error variants stable and actionable rather than collapsed into strings?

## Security and privacy

- Can any archive entry path, symlink, or race escape the staging or installation root?
- Is the manifest signature verified before any of its fields are trusted, and the archive checksum verified before extraction?
- Are limits (size, timeout) enforced before allocation or content reads?
- Can logs, CLI output, or error messages disclose a signing key, a credentialed URL, or a full local path?
- Is `update.lock` held for the whole transaction it needs to serialise, not released early?

## Architecture and Rust

- Is the one-way `updater-core` → `{updater-launcher, updater-engine}` → `updater-cli` dependency direction preserved?
- Does every project type conversion use `From` or `TryFrom`, without ad hoc conversion helpers or unchecked semantic casts?
- Is the change free of an async runtime introduced without a recorded decision in the decisions log?
- Is production code free of unsafe, panic shortcuts, debug output, and broad lint suppressions?

## Tests

- Is there evidence of the right test at the right layer, including the failure path and prohibited side effects (pointer/state left unchanged)?
- Are filesystem and archive semantics tested with real isolated temporary directories, not mocks?
- Is a platform-specific claim (Windows locking, POSIX symlinks) backed by a test that actually runs on that platform, or clearly marked as pending it?
- Would the test fail if the new production behaviour were removed?

If there are no findings, state that explicitly and list residual untested risks. Do not inflate style preferences into defects.

Before concluding, consider the change holistically. Do not accept a narrow local improvement that transfers risk or complexity into security, another crate, another platform, operations, users, tests, or future maintenance.
