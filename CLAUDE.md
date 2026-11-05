@AGENTS.md

# Auto-Updater (`auto-updater`)

Rust workspace implementing a secure, cross-platform, versioned release-set installer, launcher, and self-update mechanism for a multi-executable Rust product, per the specification. Cargo workspace of library crates plus one thin CLI binary.

**`AGENTS.md` is the binding contract for every substantive task in this repository. Read it before changing anything; nothing in this file weakens it.** The `.claude/` directory holds the detailed rules, workflows, review lenses, and templates it references, applied automatically through Claude Code's skills, subagents, and hooks; Codex sessions read the same files directly as a secondary, occasional surface, in place of skills and subagents it has no equivalent for.

## Authoritative inputs

Product behaviour is specified outside this repository. `CLAUDE.local.md` records the exact resource locations. Read the governing section before changing product behaviour; if the sources are unavailable, record the uncertainty and ask; never invent missing semantics.

## Commands

| Purpose | Command |
| --- | --- |
| Completion gate | `just ci` |
| Guidance checks plus gate | `.claude/scripts/check.sh` |
| Targeted red/green test | `cargo test -p <crate> <test_name> -- --exact` |
| Run the CLI against a disposable install root | `UPDATER_INSTALL_ROOT=<dir> cargo run -p updater-cli -- update diagnostics` |

Never claim a check passed unless it actually ran and succeeded. Report any skipped check and the reason.

## Architecture overview

- One-way workspace; every dependency edge points towards `updater-core`. The decisions log has the full crate-boundary rationale, if you need more than the summary below.
- `updater-core` owns manifest and release metadata types, local state, validated/confined path types, error types, and the atomic-write and activation-port traits. No other workspace crate dependency.
- `updater-launcher` resolves the active release set and starts the requested command's executable from it; it never activates a release or deletes one.
- `updater-engine` owns the full update transaction: manifest retrieval and signature verification, download, checksum verification, extraction, bundle and executable validation, promotion, pointer activation, rollback, cleanup, state management, and the update lock.
- `updater-cli` composes `updater-launcher` and `updater-engine` behind the `update`/`rollback`/ `install`/`update diagnostics` subcommands from the specification's "Update Commands" section, and ships a thin `updater` binary for direct use and manual/integration testing.
- Every mutation to the active pointer, `previous`, or `state.json` flows through one atomic-write path: validate -> acquire `update.lock` -> temp write + flush + atomic rename/replace -> release lock. A completed activation is the contract; a locked old executable defers cleanup, it never turns a successful activation into a reported failure.
- This is a synchronous, sequential CLI: no async runtime.

## Non-negotiables (summary; `AGENTS.md` is authoritative)

- Trait-first design and test-driven delivery: observe the failing test for the intended reason before writing production behaviour. Bug fixes require a regression test.
- No `unsafe`, `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` in production paths. One focused `thiserror` enum per crate with stable, matchable error variants.
- Every project type conversion is `From<T>` or `TryFrom<T>`; no free-form conversion helpers (`to_*`, `from_*`, `convert_*`, `parse_*`).
- Writes are atomic, lock-aware, and root-confined. Symlinks, Windows paths and file locking, and external modification of the installation directory are security boundaries.
- A release set activates only after every declared executable validates; a single failure rejects the whole set.
- `tracing` only, never `println!`; no signing keys, credentialed URLs, or full local install paths at INFO and above. Reject malformed or unknown input; never silently repair.
- Australian English spelling and punctuation for operator-facing text and documentation, including the Oxford comma.
- Markdown prose is not hard-wrapped: one line per paragraph. Wrap only where it adds real value (tables, code samples, fixed-width content); 120 characters if a hard limit is genuinely needed, e.g. wrap Rustdoc at 120 characters.
- Keep changes scoped to the requested problem.
- Do not commit, push, or rewrite history unless explicitly asked.
- Never add attributions (such as `Co-Authored-By`) or product or service references to commits.

## Rule routing

| When the task touches | Read |
| --- | --- |
| Any production behaviour | `.claude/rules/testing.md`, `.claude/workflows/tdd.md`, `.claude/workflows/change-loop.md` |
| Crate boundaries, traits, or the write pipeline | `.claude/rules/architecture.md` |
| The manifest, `release.json`, `state.json`, or CLI command contracts | `.claude/rules/release-contract.md` |
| Paths, archives, signatures, locking, or secrets | `.claude/rules/security.md` |
| Rust code, dependencies, errors, or logging | `.claude/rules/rust-quality.md` |
| Completion or CI | `.claude/workflows/quality-gate.md` |
| Session memory, diaries, or the vault knowledge store | `.claude/rules/memory.md` |

Source-of-truth precedence: explicit user instruction → `AGENTS.md` → the specification → the decisions log → `.claude` guidance → existing code and tests. When authoritative sources conflict, stop that design decision, show the exact conflict, and ask.

## Claude Code project surface

- Slash commands: `/brief` (change brief), `/tdd` (red-green-refactor loop), `/gate` (completion gate with evidence report), `/review-code` and `/review-spec` (the two `.claude` review lenses), and `/matrix` (requirement-to-test matrix).
- Subagents: `code-reviewer` and `spec-reviewer` apply the corresponding `.claude/reviewers/` lenses; use them proactively before declaring any behaviour change complete.
- Hooks: after every edit to a `.rs` file, `rustfmt --check` runs automatically and reports drift; a Markdown edit is checked for trailing whitespace the same way. A `git commit`/`gh pr create` carrying attribution text is blocked before it runs. Fix drift with `cargo fmt --all` rather than accumulating it for the gate.

## Memory and session records

`.claude/rules/memory.md` governs the three knowledge stores; in short:

- `docs/` holds example configuration, man pages, and auto-generated Rustdoc, nothing else. Neither the specification nor the decisions log lives in this repository; both are maintained in the vault.


## Repository gotchas

- `.claude/scripts/check.sh` validates every relative Markdown link in the repository and rejects trailing whitespace in guidance files; keep new documentation clean or the gate fails.
- The specification uses `<tool>`/`<Tool>` as placeholders for the eventual product command names. Do not silently pick a real name while scaffolding or writing domain-agnostic code; that choice belongs to whichever product embeds this workspace, or to an explicit, recorded decision if this workspace ever ships its own branded binary.
