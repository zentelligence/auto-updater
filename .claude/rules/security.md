# Security and Data-Integrity Rules

Assume the update manifest, `release.json`, every archive entry, and the operator's own configuration and environment variables are untrusted until validated.

## Path confinement

- Reject an archive entry path that is absolute, contains a `..` traversal component, or otherwise resolves outside the staging directory it is being extracted into (the specification's "Security Requirements" and "Bundle Validation"). This check happens before the entry is written, not after.
- A validated, confined path type is the sole boundary for turning an archive entry name or a `release.json` command path into something that touches the filesystem. Never use lexical `starts_with` on unnormalised input as the check.
- Resolve the installation root, `staging/`, and `downloads/` consistently, including intermediate symlinks, before comparing a target path against them.
- Revalidate confinement at the point of extraction or promotion, not only when the path was first parsed, to reduce check/use races.

## Verification before trust

- Retrieve the update manifest and release archives over HTTPS only.
- Verify a cryptographic signature over the exact manifest bytes before using any field from it (target list, URLs, checksums, version). A signature failure aborts before any of those fields are trusted for anything, including logging them unredacted.
- Verify the SHA-256 of a downloaded archive before extracting it. A mismatch deletes the download and aborts; it does not retry silently or fall back to an unverified extraction.
- Match the selected asset to the actual detected target triple, not an operator-supplied one, unless an explicit override is a documented, separate code path.
- Never activate a partially downloaded, partially extracted, or partially validated release set.

## Persistence

- Write a uniquely named temporary file in the target directory, flush it as the platform requires, then atomically rename or replace the target. This applies to the active pointer, `previous`, `state.json`, and nothing else writes these files.
- `update.lock` serialises the whole check/download/verify/activate/rollback/cleanup transaction. Concurrent update attempts must not corrupt the pointer or `state.json`; one loses the race cleanly, it does not partially apply.
- Deletion of a superseded release set is best-effort: if a file is locked (most commonly a running `.exe` on Windows), retain that release set and retry later. Never force-delete a locked executable.

## Secrets and exposure

- Keep private signing keys out of the repository and out of any release artefact; only public keys ship with the client.
- Embed more than one trusted public key where key rotation is in scope, and check a signing-key identifier in the detached signature metadata rather than assuming a single fixed key forever.
- Never log a signing key, a download URL with embedded credentials, or a full local installation path at INFO level or above.
- Test redaction and rejection behaviour, including the error paths, not only the success path.

Security-relevant defects require a regression test at the lowest affected layer (usually `updater-core` for parsing/confinement, `updater-engine` for verification and activation) and, where practical, an end-to-end rejection test through the `updater-cli` subcommand.
