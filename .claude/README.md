# Claude Development Kit

This directory is the project's primary operating handbook. The root [AGENTS.md](../AGENTS.md) is the mandatory engineering contract for every coding agent; everything here supplies the focused rules, workflows, review lenses, and templates it references, plus the Claude Code-native mechanisms (skills, subagents, and hooks) that apply them automatically during a session. Codex sessions are a secondary, occasional surface: they read `AGENTS.md` and the same plain-markdown rule files directly, without the Claude-specific frontmatter in `commands/` and `agents/` meaning anything to them.

## Layout

```text
.claude/
├── README.md
├── settings.json
├── agents/
│   ├── code-reviewer.md
│   └── spec-reviewer.md
├── commands/
│   ├── brief.md
│   ├── gate.md
│   ├── matrix.md
│   ├── review-code.md
│   ├── review-spec.md
│   └── tdd.md
├── hooks/
│   ├── markdown-check.sh
│   ├── no-commit-attribution.sh
│   ├── no-tracking-refs.sh
│   ├── rustdoc-width-check.sh
│   └── rustfmt-check.sh
├── rules/
│   ├── 00-index.md
│   ├── architecture.md
│   ├── memory.md
│   ├── memory.local.md
│   ├── release-contract.md
│   ├── rust-quality.md
│   ├── security.md
│   └── testing.md
├── workflows/
│   ├── change-loop.md
│   ├── quality-gate.md
│   └── tdd.md
├── reviewers/
│   ├── code-review.md
│   └── specification-review.md
├── templates/
│   ├── change-brief.md
│   └── requirement-test-matrix.md
└── scripts/
    └── check.sh
```

## How Claude Code uses this

- **Skills** (`commands/`) are the slash commands `/brief`, `/gate`, `/matrix`, `/review-code`, `/review-spec`, and `/tdd`; each loads the relevant `rules/`, `workflows/`, `reviewers/`, or `templates/` file before acting.
- **Subagents** (`agents/`) — `code-reviewer` and `spec-reviewer` — apply the corresponding `reviewers/` lens to a diff and never modify code themselves.
- **Hooks** (`hooks/`, wired in `settings.json`) run automatically after every `Edit`/`Write`: `rustfmt-check.sh` reports Rust formatting drift, `rustdoc-width-check.sh` reports Rustdoc lines over 120 characters, `markdown-check.sh` reports trailing whitespace, and `no-tracking-refs.sh` reports an `R-*`/`D-*`/`Q-*`/`VD-*`/"phase N" tracking-identifier reference in code or docs.`no-commit-attribution.sh` blocks a `git commit`/`gh pr create` carrying attribution text before it runs.
- **`scripts/check.sh`** is the completion gate: guidance checks (required files, trailing whitespace, local Markdown links, shellcheck) followed by `just ci` once the Cargo workspace builds.

## Session entry point

1. Read `AGENTS.md` at the repository root.
2. Read [`rules/00-index.md`](rules/00-index.md).
3. Read the governing section of the specification (`CLAUDE.local.md` has its exact location).
4. Load only the rules and workflow needed for the task.
5. Use the review lenses before reporting completion.

## Codex sessions (occasional)

Codex has no equivalent of skills or subagents, so it reads the same files as plain reference: `AGENTS.md` first, then `rules/00-index.md`, following the same rule routing everything else in this kit uses. `commands/*.md` and `agents/*.md` still work as reference documentation even though Codex cannot invoke them as commands or delegate to them as subagents.

No repository-local `config.toml` is supplied. Model selection, trust, sandbox, approval policy, external MCP servers, and credentials belong to the operator's own agent configuration, not version control.

## Maintenance policy

- Put rules shared by every coding agent in `AGENTS.md`.
- Put focused, task-scoped execution guidance here.
- Put stable product and operator documentation in the root `README.md` or `docs/`.
- Keep these files concise and point to the authoritative specification instead of copying it.
- Change guidance in the same change as the behaviour or failure mode that made it necessary.
