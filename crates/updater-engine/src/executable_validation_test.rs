use std::collections::HashMap;

use tempfile::tempdir;
use updater_core::ReleaseMetadata;

use super::*;

struct FakeProbe(HashMap<&'static str, &'static str>);

impl ExecutableVersionProbe for FakeProbe {
    fn probe_version(&self, command_name: &str, _executable: &Path) -> Result<ProductVersion, EngineError> {
        let reported = self
            .0
            .get(command_name)
            .ok_or_else(|| EngineError::ExecutableProbeFailed {
                name: command_name.to_string(),
                reason: "no fixture registered".to_string(),
            })?;
        Ok(ProductVersion::try_from(*reported)?)
    }
}

fn release_json() -> &'static str {
    r#"{
      "schema_version": 1,
      "product": "tool",
      "version": "1.10.0",
      "target": "aarch64-apple-darwin",
      "published_at": "2026-09-21T04:00:00Z",
      "minimum_launcher_version": "1.0.0",
      "commands": [
        { "name": "tool", "path": "bin/tool" },
        { "name": "tool-agent", "path": "bin/tool-agent" }
      ]
    }"#
}

#[test]
fn accepts_executables_that_report_the_expected_version() {
    let temp = tempdir().unwrap();
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    let probe = FakeProbe(HashMap::from([("tool", "1.10.0"), ("tool-agent", "1.10.0")]));

    validate_executable_versions(&probe, &metadata, temp.path()).expect("versions match");
}

#[test]
fn rejects_an_executable_reporting_a_mismatched_version() {
    let temp = tempdir().unwrap();
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    let probe = FakeProbe(HashMap::from([("tool", "1.10.0"), ("tool-agent", "1.9.0")]));

    let error = validate_executable_versions(&probe, &metadata, temp.path()).unwrap_err();

    assert!(matches!(error, EngineError::ExecutableVersionMismatch { name, .. } if name == "tool-agent"));
}

#[test]
fn propagates_a_probe_failure() {
    let temp = tempdir().unwrap();
    let metadata = ReleaseMetadata::try_from(release_json().as_bytes()).unwrap();
    let probe = FakeProbe(HashMap::new());

    let error = validate_executable_versions(&probe, &metadata, temp.path()).unwrap_err();

    assert!(matches!(error, EngineError::ExecutableProbeFailed { .. }));
}

#[cfg(unix)]
#[test]
fn spawn_probe_reports_a_failure_for_a_nonexistent_executable() {
    let probe = SpawnVersionProbe;

    let error = probe
        .probe_version("tool", Path::new("/no/such/executable"))
        .unwrap_err();

    assert!(matches!(error, EngineError::ExecutableProbeFailed { .. }));
}
