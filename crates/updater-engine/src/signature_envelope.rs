use crate::error::EngineError;

/// The detached-signature envelope fetched alongside the update manifest (conventionally at
/// `<manifest-url>.sig`): a small, self-describing JSON document naming which trusted key signed
/// the manifest, so a rotated key can be identified without guessing.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignatureEnvelope {
    pub key_id: String,
    signature: String,
}

impl SignatureEnvelope {
    pub fn parse(bytes: &[u8]) -> Result<Self, EngineError> {
        serde_json::from_slice(bytes).map_err(|source| EngineError::InvalidSignatureEnvelope {
            reason: source.to_string(),
        })
    }

    pub fn signature_bytes(&self) -> Result<Vec<u8>, EngineError> {
        decode_hex(&self.signature)
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, EngineError> {
    if !value.len().is_multiple_of(2) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(EngineError::InvalidSignatureEnvelope {
            reason: format!("{value:?} is not a valid hex-encoded signature"),
        });
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|source| EngineError::InvalidSignatureEnvelope {
                reason: source.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "signature_envelope_test.rs"]
mod tests;
