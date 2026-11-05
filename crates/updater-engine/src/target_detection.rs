use updater_core::TargetTriple;

use crate::error::EngineError;

/// Detects the Rust target triple for the platform this binary was compiled for.
///
/// Uses compile-time `std::env::consts` values rather than runtime introspection, since a release
/// binary's own target triple is fixed at build time; this must never be overridden by an
/// operator-supplied value except through an explicit, separately documented override path.
pub fn detect_target() -> Result<TargetTriple, EngineError> {
    let triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("windows", "aarch64") => "aarch64-pc-windows-msvc",
        (os, arch) => {
            return Err(EngineError::UnsupportedHostPlatform {
                os: os.to_string(),
                arch: arch.to_string(),
            });
        }
    };
    Ok(TargetTriple::try_from(triple)?)
}

#[cfg(test)]
#[path = "target_detection_test.rs"]
mod tests;
