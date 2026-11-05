use ed25519_dalek::{Signature, VerifyingKey};

use crate::error::EngineError;

/// A trusted public signing key, identified by the short key id embedded in the detached
/// signature's metadata. Embedding more than one lets the workspace support key rotation.
#[derive(Debug, Clone)]
pub struct TrustedKey {
    key_id: String,
    verifying_key: VerifyingKey,
}

impl TrustedKey {
    pub fn new(key_id: impl Into<String>, public_key_bytes: &[u8; 32]) -> Result<Self, EngineError> {
        let key_id = key_id.into();
        let verifying_key =
            VerifyingKey::from_bytes(public_key_bytes).map_err(|source| EngineError::InvalidTrustedKey {
                key_id: key_id.clone(),
                reason: source.to_string(),
            })?;
        Ok(TrustedKey { key_id, verifying_key })
    }
}

/// Verifies a detached signature over exact manifest bytes, before any field of the manifest is
/// trusted. Injectable so tests can supply a deterministic key pair instead of hitting real
/// signing infrastructure.
pub trait ManifestVerifier {
    fn verify(&self, manifest_bytes: &[u8], signature_bytes: &[u8], key_id: &str) -> Result<(), EngineError>;
}

/// The real Ed25519 verifier, checking against a fixed set of trusted public keys.
pub struct Ed25519ManifestVerifier {
    trusted_keys: Vec<TrustedKey>,
}

impl Ed25519ManifestVerifier {
    pub fn new(trusted_keys: Vec<TrustedKey>) -> Self {
        Ed25519ManifestVerifier { trusted_keys }
    }
}

impl ManifestVerifier for Ed25519ManifestVerifier {
    fn verify(&self, manifest_bytes: &[u8], signature_bytes: &[u8], key_id: &str) -> Result<(), EngineError> {
        let key = self
            .trusted_keys
            .iter()
            .find(|key| key.key_id == key_id)
            .ok_or_else(|| EngineError::UnknownSigningKey {
                key_id: key_id.to_string(),
            })?;

        let signature = Signature::try_from(signature_bytes).map_err(|_| EngineError::ManifestSignatureInvalid)?;

        key.verifying_key
            .verify_strict(manifest_bytes, &signature)
            .map_err(|_| EngineError::ManifestSignatureInvalid)
    }
}

/// Signs `message` with `signing_key_bytes`, for use in fixtures and tests that need a real,
/// verifiable signature without depending on production signing infrastructure.
#[cfg(test)]
pub(crate) fn sign_for_test(signing_key_bytes: &[u8; 32], message: &[u8]) -> Vec<u8> {
    use ed25519_dalek::Signer;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(signing_key_bytes);
    signing_key.sign(message).to_bytes().to_vec()
}

#[cfg(test)]
#[path = "signature_test.rs"]
mod tests;
