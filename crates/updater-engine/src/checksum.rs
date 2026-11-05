use std::io::Read;

use sha2::{Digest, Sha256};
use updater_core::Sha256Digest;

use crate::error::EngineError;

/// Computes the SHA-256 digest of a byte-streaming reader without loading the whole file into
/// memory at once.
pub fn compute_sha256(mut reader: impl Read, source_path: &std::path::Path) -> Result<Sha256Digest, EngineError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|source| EngineError::ChecksumIo {
            path: source_path.to_path_buf(),
            source,
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(Sha256Digest::try_from(hex_encode(&digest))?)
}

/// Verifies that `actual` matches `expected`, comparing the full digest rather than short-circuiting.
pub fn verify_sha256(expected: &Sha256Digest, actual: &Sha256Digest) -> Result<(), EngineError> {
    if expected.as_str() == actual.as_str() {
        Ok(())
    } else {
        Err(EngineError::ChecksumMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // A `Write` impl over a `String` never fails.
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
#[path = "checksum_test.rs"]
mod tests;
