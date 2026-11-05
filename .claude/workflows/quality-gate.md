# Quality Gate

Verification is proportional during development and comprehensive before a change is declared complete.

Passing commands is necessary but not sufficient. Completion also requires a holistic review of correctness, security, data integrity, architecture, cross-platform behaviour, performance, operability, maintainability, and documentation wherever the change can affect them.

## During the loop

Run the smallest target that demonstrates red or green, for example:

```sh
cargo test -p updater-core release_json_rejects_unknown_schema_version -- --exact
cargo test -p updater-engine archive_extraction_rejects_parent_traversal -- --exact
```

Use the actual test target and name; do not cargo-cult these examples.

## Before completion

If the repository supplies `just ci`, it is the authoritative completion gate. Otherwise run:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit
```

Also run any applicable platform suite or manual acceptance procedure from the specification's "Test Plan" section. Documentation-only work must at least run `.claude/scripts/check.sh` and inspect rendered links.

## Versioning

Bump `[workspace.package] version` in the root `Cargo.toml` as part of closing any change that adds, removes, or alters a CLI command, on-disk format, or other operator-visible behaviour; a documentation-only or purely internal refactor does not need a bump. While the project remains pre-1.0, bump MINOR for a completed, behaviour-visible change and PATCH for a fix within already-shipped behaviour. Move to `1.0.0` only once a recorded decision says the workspace has reached the stability the specification's "Acceptance Criteria" section describes, not merely once Linux CI is green.

## Completion evidence

The final report distinguishes:

- checks that passed;
- checks that failed and why;
- checks not run and why; and
- manual or platform-specific validation still required.

Warnings, flaky reruns, ignored tests, and unavailable audit tooling are not a clean pass. Never claim a platform-wide result from a single-OS test run.
