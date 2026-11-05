use std::time::Duration;

use crate::error::EngineError;

/// The largest response this workspace will read into memory for a manifest or checksum-style
/// document. Archive downloads use [`crate::download::download_to_file`], which streams to disk
/// instead and is not subject to this in-memory limit.
pub const MAX_DOCUMENT_BYTES: u64 = 10 * 1024 * 1024;

/// An injectable HTTPS transport, so manifest and archive retrieval can be tested against a local
/// fixture instead of the live network.
pub trait HttpTransport {
    /// Fetches the bytes at `url` into memory, enforcing HTTPS and `max_bytes`.
    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, EngineError>;

    /// Streams the response at `url` to `writer`, enforcing HTTPS, returning the number of bytes
    /// written.
    fn stream(&self, url: &str, writer: &mut dyn std::io::Write) -> Result<u64, EngineError>;
}

/// The real HTTPS transport, backed by `ureq`.
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl Default for UreqTransport {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build();
        UreqTransport { agent: config.into() }
    }
}

fn require_https(url: &str) -> Result<(), EngineError> {
    if url.starts_with("https://") {
        Ok(())
    } else {
        Err(EngineError::InsecureManifestUrl { url: url.to_string() })
    }
}

impl HttpTransport for UreqTransport {
    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, EngineError> {
        require_https(url)?;
        let mut response = self.agent.get(url).call().map_err(|source| EngineError::Transport {
            url: url.to_string(),
            reason: source.to_string(),
        })?;
        response
            .body_mut()
            .with_config()
            .limit(max_bytes)
            .read_to_vec()
            .map_err(|source| EngineError::Transport {
                url: url.to_string(),
                reason: source.to_string(),
            })
    }

    fn stream(&self, url: &str, writer: &mut dyn std::io::Write) -> Result<u64, EngineError> {
        require_https(url)?;
        let mut response = self.agent.get(url).call().map_err(|source| EngineError::Transport {
            url: url.to_string(),
            reason: source.to_string(),
        })?;
        let mut reader = response.body_mut().as_reader();
        std::io::copy(&mut reader, writer).map_err(|source| EngineError::Transport {
            url: url.to_string(),
            reason: source.to_string(),
        })
    }
}

#[cfg(test)]
#[path = "transport_test.rs"]
mod tests;
