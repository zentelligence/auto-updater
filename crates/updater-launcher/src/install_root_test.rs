use std::collections::HashMap;
use std::path::Path;

use super::*;

/// An absolute path on the host running the test, which is what an installation root must be.
const CUSTOM_ROOT: &str = if cfg!(windows) {
    "C:\\custom\\path"
} else {
    "/custom/path"
};

struct FakeEnvironment(HashMap<String, String>);

impl EnvironmentLookup for FakeEnvironment {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

fn fake_env(pairs: &[(&str, &str)]) -> FakeEnvironment {
    FakeEnvironment(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect())
}

#[test]
fn override_variable_takes_precedence() {
    let env = fake_env(&[("TOOL_INSTALL_ROOT", CUSTOM_ROOT)]);

    let root = resolve_installation_root(&env, "tool", "tool").unwrap();

    assert_eq!(root.as_path(), Path::new(CUSTOM_ROOT));
}

#[test]
fn override_variable_name_is_upper_cased() {
    let env = fake_env(&[("TOOL_INSTALL_ROOT", CUSTOM_ROOT)]);

    let root = resolve_installation_root(&env, "tool", "tool").unwrap();

    assert_eq!(root.as_path(), Path::new(CUSTOM_ROOT));
}

#[test]
fn rejects_a_relative_override_path() {
    let env = fake_env(&[("TOOL_INSTALL_ROOT", "relative/path")]);

    let error = resolve_installation_root(&env, "tool", "tool").unwrap_err();

    assert!(matches!(error, LauncherError::Core(_)));
}

#[cfg(unix)]
#[test]
fn falls_back_to_home_local_share_on_unix() {
    let env = fake_env(&[("HOME", "/home/user")]);

    let root = resolve_installation_root(&env, "tool", "tool").unwrap();

    assert_eq!(root.as_path(), Path::new("/home/user/.local/share/tool"));
}

#[cfg(unix)]
#[test]
fn fails_when_home_is_unavailable_on_unix() {
    let env = fake_env(&[]);

    let error = resolve_installation_root(&env, "tool", "tool").unwrap_err();

    assert!(matches!(error, LauncherError::HomeDirectoryUnavailable));
}
