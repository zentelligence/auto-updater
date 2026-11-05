# Requirement-to-Test Matrix: <feature or phase>

Keep rows at observable-contract granularity, referencing specification section headings since there
is no numbered requirement scheme. Link to tests once paths exist.

| Specification section | Contract/example | Test layer | Test name/path | Negative or platform cases | Status |
| --- | --- | --- | --- | --- | --- |
| `<e.g. "Atomic Activation">` | `<observable behaviour>` | unit/property/integration/release-contract/platform/acceptance | `<test>` | `<cases>` | red/green/pending |

## Error coverage

| Specification "Failure Handling" row | No-change assertion | Test | Status |
| --- | --- | --- | --- |
| `<failure>` | `<state preserved>` | `<test>` | red/green/pending |

## Gate evidence

| Acceptance criterion | Automated evidence | Manual evidence | Remaining limitation |
| --- | --- | --- | --- |
| `<specification "Acceptance Criteria" item>` | `<command/test>` | `<procedure/result>` | `<limitation or none>` |
