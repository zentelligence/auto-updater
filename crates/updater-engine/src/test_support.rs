//! Shared fakes for deterministic, offline tests across `updater-engine`'s modules. Test-only;
//! never compiled into production code.

use std::collections::HashMap;
use std::io::Write;

use crate::clock::Clock;
use crate::error::EngineError;
use crate::transport::HttpTransport;

pub struct FixedClock(pub &'static str);

impl Clock for FixedClock {
    fn now_rfc3339(&self) -> String {
        self.0.to_string()
    }
}

/// An in-memory `HttpTransport` fixture: no real network access, ever.
#[derive(Default)]
pub struct FakeTransport {
    responses: HashMap<String, Vec<u8>>,
}

impl FakeTransport {
    pub fn with_response(mut self, url: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        self.responses.insert(url.into(), bytes.into());
        self
    }
}

impl HttpTransport for FakeTransport {
    fn fetch(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, EngineError> {
        if !url.starts_with("https://") {
            return Err(EngineError::InsecureManifestUrl { url: url.to_string() });
        }
        let bytes = self.responses.get(url).cloned().ok_or_else(|| EngineError::Transport {
            url: url.to_string(),
            reason: "no fixture response registered".to_string(),
        })?;
        if bytes.len() as u64 > max_bytes {
            return Err(EngineError::ResponseTooLarge {
                url: url.to_string(),
                limit: max_bytes,
            });
        }
        Ok(bytes)
    }

    fn stream(&self, url: &str, writer: &mut dyn Write) -> Result<u64, EngineError> {
        if !url.starts_with("https://") {
            return Err(EngineError::InsecureManifestUrl { url: url.to_string() });
        }
        let bytes = self.responses.get(url).cloned().ok_or_else(|| EngineError::Transport {
            url: url.to_string(),
            reason: "no fixture response registered".to_string(),
        })?;
        writer.write_all(&bytes).map_err(|source| EngineError::Transport {
            url: url.to_string(),
            reason: source.to_string(),
        })?;
        Ok(bytes.len() as u64)
    }
}
