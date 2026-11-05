# Change Loop

## Discover

1. Inspect repository status and nearby code/tests.
2. Read the governing specification section and any relevant decisions log entry.
3. Write a concise change brief: in scope, out of scope, risks, and verification.
4. Resolve contract ambiguity before it reaches `release.json`, the manifest, `state.json`, or the CLI surface.
5. Assess system-wide effects: security, data integrity, architecture, cross-platform behaviour, performance, operations, maintainability, and docs.

## Design

1. Place behaviour in `updater-core`, `updater-launcher`, or `updater-engine`; keep `updater-cli` translation-only.
2. Identify security boundaries, failure policy, and interruption safety.
3. List the test layers needed. Start with one red example.
4. Prefer the smallest complete change that leaves a coherent, production-grade public contract. Small scope never excuses incomplete quality.

## Implement

1. Follow the red-green-refactor loop.
2. Keep CLI handlers thin and errors typed.
3. Run targeted tests frequently.
4. Inspect diffs for accidental scope, generated noise, debug output, and secret exposure.

## Verify and report

1. Run the applicable quality gate.
2. Review through both `reviewers/code-review.md` and `reviewers/specification-review.md` when product behaviour changes.
3. Report changed files, behaviour delivered, tests/checks run, and any unverified platform or manual acceptance work.

Do not opportunistically implement later scope. Record useful follow-ups in the decisions log without broadening the current change.
