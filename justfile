set shell := ["bash", "-euo", "pipefail", "-c"]

fmt:
    cargo fmt --all --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

audit:
    cargo audit

# Regenerate THIRD_PARTY_LICENSES.md after any dependency change.
licences:
    cargo about generate --locked --fail about.hbs -o THIRD_PARTY_LICENSES.md

# Fail if a dependency is under an unaccepted licence or THIRD_PARTY_LICENSES.md has drifted.
licences-check:
    generated="$(mktemp)"; trap 'rm -f "${generated}"' EXIT; \
    cargo about generate --locked --fail about.hbs -o "${generated}"; \
    diff -u THIRD_PARTY_LICENSES.md "${generated}" \
      || { echo 'THIRD_PARTY_LICENSES.md is stale; run `just licences`' >&2; exit 1; }

ci: fmt clippy test audit licences-check
    shellcheck .claude/scripts/check.sh .claude/hooks/rustfmt-check.sh .claude/hooks/markdown-check.sh \
      .claude/hooks/no-commit-attribution.sh .claude/hooks/mempalace-reminder.sh
