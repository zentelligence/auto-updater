---
description: Check behaviour, formats, and delivery claims against the specification
argument-hint: <specification section(s), CLI commands, or a feature>
---

Run a specification conformance review of: $ARGUMENTS

1. Use the `spec-reviewer` subagent to apply `.claude/reviewers/specification-review.md`, reading the specification and the decisions log directly. If they are unreadable, report that and stop; never review against remembered or invented semantics.
2. Relay each gap classified as violation, missing, drift, ambiguity, or future scope, with the affected specification section(s) and the evidence.
3. For ambiguities, present the alternatives and compatibility consequences; never resolve one by silently choosing the easiest implementation.
4. State whether any claimed acceptance criterion from the specification is fully satisfied, and which ones lack evidence.
