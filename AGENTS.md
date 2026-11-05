# Updater Agent Contract

This file governs all work in this repository. More specific `AGENTS.md` files may add constraints for their subtree, but may not weaken this contract.

## Mission and current state

Build `updater`, a Rust workspace that gives a multi-executable Rust product a secure, cross-platform, versioned release-set installer, launcher, and self-update mechanism. Scaffolding is complete; no domain behaviour is implemented yet. Consult the decisions log for the boundary and scope decisions made while scaffolding, and extend it as further decisions are made.

The authoritative product input (the specification) is maintained outside this repository. See `CLAUDE.local.md` for the exact resource location if present in your checkout, or ask the repository owner. `docs/` in this repository holds only example configuration, man pages, and auto-generated Rustdoc for the product this workspace ships.

Read the specification before changing product behaviour. If it is unavailable in your environment, do not invent missing semantics; record the uncertainty and ask for the required source.

## Quality mandate

**Holistic quality trumps everything else.** Optimise for the integrity of the whole product and its lifecycle, including correctness, security, data safety, architecture, interoperability, test strength, operability, maintainability, performance, accessibility of diagnostics, and documentation. Suspend any "simplest and smallest bias".

Delivery speed, diff size, convenience, token cost, and passing a narrow test are never sufficient reasons to weaken the result. Scope discipline still applies: solve the requested problem completely and coherently without adding unrequested product surface. However, any bias towards simplest and or smallest is explicitly overruled as violation of the quality mandate.

Prior existence of an error, deviation, or violation is no excuse for perpetuating or not addressing it. Identified issues are not "raised for later"; the only permitted exception is a change with significant architectural implications that would require a multi-stage analysis, spec design, work breakdown plan, along with one or more full critique and refinement passes.

## Start every substantive task

1. Read this file and [`.claude/rules/00-index.md`](.claude/rules/00-index.md).
2. Inspect the working tree and preserve unrelated or user-authored changes.
3. Identify the governing section of the specification, so the change is grounded in what the product must do.
4. Read only the `.claude` rules relevant to the task.
5. Define the smallest observable behaviour that can be driven by a failing test.

## Non-negotiable engineering rules

- Follow trait-first development.
- Use test-driven delivery for every behaviour change: red, green, then refactor. See [`.claude/workflows/tdd.md`](.claude/workflows/tdd.md).
- Do not add production behaviour until a focused test has failed for the expected reason. Bug fixes require a regression test.
- Keep dependency direction one-way: `updater-launcher`, `updater-engine`, and `updater-cli` depend on `updater-core`; `updater-cli` depends on `updater-launcher` and `updater-engine`. Nothing depends back "up" the chain, and `updater-core` depends on nothing else in the workspace.
- Every path taken from an archive entry, a manifest, or the installation root must become a validated, confined type before it touches the filesystem. Never accept a raw `Path` or `PathBuf` straight from untrusted input at a domain or service boundary.
- All activation and state writes (the active pointer, `previous`, `state.json`) pass through one atomic write path: unique temp file in the target directory, flushed, then an atomic rename or platform-safe replace. No code writes these files directly outside that path.
- Production code contains no `unsafe`, `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!`. Use typed errors and preserve stable, matchable error variants.
- Implement every project type conversion with `From<T>` when infallible or `TryFrom<T>` when fallible. Never create free-form conversion functions. Callers may use the corresponding `Into` or `TryInto` blanket traits.
- This is a synchronous, sequential CLI, not a server: default to blocking I/O and the standard library's or a chosen HTTP client's blocking mode. Do not introduce an async runtime without a specific, documented need that blocking code cannot meet.
- Writes are atomic, lock-aware (`update.lock`), and confined to the resolved installation root. Treat symlink handling, Windows path and file-locking behaviour, and external modification of the installation directory as security boundaries, not edge cases.
- A release set is validated in full before activation; a failure in any declared executable rejects the whole release set, never a partial activation.
- Use `tracing`, never `println!` or `eprintln!`, for runtime diagnostics. Never log signing keys, download URLs with embedded credentials, or full local installation paths at INFO or above.
- Reject malformed or unknown manifest, `release.json`, or `state.json` fields. Never silently repair or degrade a write.
- Use Australian English spelling, grammar, and punctuation for operator-facing text and documentation, including the Oxford comma.
- Maximum 1000 lines per code file.
- Do not hard-wrap Markdown prose: one line per paragraph, soft-wrapped by the reader's editor. Wrap only where it adds real value (tables, code samples, or content that must align in a fixed-width view); where a hard limit is genuinely needed or adopted.
- Rustdoc wraps at 120 not 80 characters.
- Keep changes scoped. Do not commit, push, rewrite history, or alter releases unless the user explicitly asks.
- Never add attribution to commit messages.
- Use zero-context sub-agents; forked contexts waste tokens.
- Never add 'current state' updates to repo files.

## Test standards

Tests are part of the design, not a closing activity.

- Unit tests cover pure domain rules (semantic-version comparison, manifest and `release.json` parsing, path confinement, pointer parsing) and each error branch.
- Integration tests use a real, isolated temporary installation root and, where relevant, a real local HTTP fixture for manifest and archive retrieval. Do not replace meaningful filesystem semantics with mocks.
- Platform tests cover POSIX symlink activation and Windows pointer-file replace-on-locked-executable semantics as distinct cases, not a single cross-platform abstraction assumed to behave identically; a case that can only run on one operating system is marked as such rather than skipped silently.
- Property tests cover archive path confinement, version-ordering, and other invariant-heavy behaviour.
- Tests must be deterministic. Inject clocks, HTTP transports, hashes, and signature verifiers. Never depend on network access, wall-clock sleeps, the operator's home directory, or the operator's real `PATH`.
- Do not weaken, delete, ignore, or over-broaden a test merely to make it pass.

Name tests for the behaviour they prove, not for an internal tracking identifier: a reader outside this project must be able to understand what a test guards from its name and body alone. Comments follow the same rule: explain the WHY in terms anyone reading the code can verify against the code or the specification, never by pointing at a decision or task identifier that exists only in chat history.

## Rust quality gate

Run the narrowest relevant test during the red/green loop. Before declaring a code change complete, run the repository gate:

```sh
just ci
```

Until that task exists, run:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit
```

Run [`.claude/scripts/check.sh`](.claude/scripts/check.sh) for repository guidance checks; it also runs the Rust quality gate. Never claim a check passed unless it ran successfully. Report any skipped check and its reason.

## Definition of done

A change is complete only when:

- the governing section of the specification is identified and, for a scope-shaping choice, recorded in the decisions log;
- the failing test was observed and the minimal implementation makes it pass;
- the change has been assessed holistically, not only against its focused test;
- relevant negative, security, and platform cases are covered;
- formatting, linting, tests, and dependency audit pass at the appropriate scope;
- operator-facing errors are actionable and secrets remain redacted;
- documentation and examples match behaviour; and
- the final report lists changed files, verification performed, and residual risks or deliberate follow-ups.

Never add attributions to commits.
