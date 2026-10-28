use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::Rng;
use ssh_key::{private::KeypairData, PrivateKey};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::Member;

#[derive(Clone)]
pub struct ServerIdentity {
    public_key_openssh: String,
    signing_key: SigningKey,
}

impl ServerIdentity {
    pub fn load_from_default_ssh_dir() -> Result<Self> {
        let home_dir = std::env::var("HOME").context("HOME is not set")?;
        Self::load_from_ssh_dir(&PathBuf::from(home_dir).join(".ssh"))
    }

    pub fn load_from_ssh_dir(ssh_dir: &Path) -> Result<Self> {
        let private_key_path = ssh_dir.join("id_ed25519");
        let public_key_path = ssh_dir.join("id_ed25519.pub");

        Self::load_from_files(&private_key_path, &public_key_path)
    }

    pub fn load_from_files(private_key_path: &Path, public_key_path: &Path) -> Result<Self> {
        let private_key = PrivateKey::read_openssh_file(private_key_path)
            .with_context(|| format!("failed to read {}", private_key_path.display()))?;
        let signing_key = match private_key.key_data() {
            KeypairData::Ed25519(keypair) => SigningKey::from_bytes(&keypair.private.to_bytes()),
            _ => anyhow::bail!("server private key must be an Ed25519 key"),
        };
        let public_key_openssh = fs::read_to_string(public_key_path)
            .with_context(|| format!("failed to read {}", public_key_path.display()))?
            .trim()
            .to_string();
        if !public_key_openssh.starts_with("ssh-ed25519 ") {
            anyhow::bail!("server public key must use the ssh-ed25519 format");
        }
        Ok(Self {
            public_key_openssh,
            signing_key,
        })
    }

    pub fn public_key_openssh(&self) -> &str {
        &self.public_key_openssh
    }

    pub fn sign_challenge(&self, challenge: &str) -> Result<String> {
        if challenge.trim().is_empty() {
            anyhow::bail!("server challenge must not be empty");
        }
        Ok(BASE64.encode(self.signing_key.sign(challenge.as_bytes()).to_bytes()))
    }
}

pub fn create_demo_member() -> Member {
    let signing_key = SigningKey::from_bytes(&rand::random());
    let public_key = encode_ssh_ed25519_public_key(&signing_key.verifying_key());
    println!("Demo member ID: demo-member");
    println!("Demo member public key registered in the app:\n{public_key}");
    Member {
        id: "demo-member".to_string(),
        name: "Demo Member".to_string(),
        email: "demo@harstadmakers.no".to_string(),
        public_key_pem: public_key,
        active: true,
        created_at: Utc::now().to_rfc3339(),
        expires_at: (Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
        last_used: None,
    }
}

pub fn normalize_public_key_pem(
    public_key_pem: Option<String>,
    public_key_jwk: Option<serde_json::Value>,
) -> Result<String> {
    if let Some(key) = public_key_pem {
        if key.trim().starts_with("ssh-ed25519 ") {
            return Ok(key.trim().to_string());
        }
        anyhow::bail!("invalid Ed25519 public key; expected ssh-ed25519 format");
    }
    let jwk = public_key_jwk.context("missing Ed25519 public key value")?;
    if jwk.get("kty").and_then(|v| v.as_str()) != Some("OKP")
        || jwk.get("crv").and_then(|v| v.as_str()) != Some("Ed25519")
    {
        anyhow::bail!("public key JWK must be an Ed25519 key");
    }
    let bytes = base64url_decode(
        jwk.get("x")
            .and_then(|v| v.as_str())
            .context("missing JWK field 'x'")?,
    )?;
    let raw: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Ed25519 public key must be 32 bytes"))?;
    Ok(encode_ssh_ed25519_public_key(&VerifyingKey::from_bytes(
        &raw,
    )?))
}

pub fn verify_member_signature(public_key: &str, message: &str, signature: &str) -> Result<()> {
    let payload = public_key
        .split_whitespace()
        .nth(1)
        .context("invalid Ed25519 public key")?;
    let wire = BASE64.decode(payload)?;
    let raw = wire
        .get(4 + 11 + 4..)
        .context("invalid Ed25519 public key payload")?;
    let key: [u8; 32] = raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("Ed25519 public key must be 32 bytes"))?;
    let signature = Signature::from_slice(&BASE64.decode(signature)?)?;
    VerifyingKey::from_bytes(&key)?.verify(message.as_bytes(), &signature)?;
    Ok(())
}

pub fn random_nonce() -> String {
    (0..32)
        .map(|_| rand::thread_rng().gen_range(b'a'..=b'z') as char)
        .collect()
}

fn base64url_decode(value: &str) -> Result<Vec<u8>> {
    let mut normalized = value.replace('-', "+").replace('_', "/");
    while !normalized.len().is_multiple_of(4) {
        normalized.push('=');
    }
    Ok(BASE64.decode(normalized)?)
}

fn encode_ssh_ed25519_public_key(key: &VerifyingKey) -> String {
    let algorithm = b"ssh-ed25519";
    let bytes = key.to_bytes();
    let mut wire = Vec::with_capacity(4 + algorithm.len() + 4 + bytes.len());
    wire.extend_from_slice(&(algorithm.len() as u32).to_be_bytes());
    wire.extend_from_slice(algorithm);
    wire.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    wire.extend_from_slice(&bytes);
    format!("ssh-ed25519 {} harstad-makers", BASE64.encode(wire))
}

#[cfg(test)]
pub fn generate_test_server_identity() -> ServerIdentity {
    let signing_key = SigningKey::from_bytes(&rand::random());
    ServerIdentity {
        public_key_openssh: encode_ssh_ed25519_public_key(&signing_key.verifying_key()),
        signing_key,
    }
}

#[cfg(test)]
pub fn generate_test_member_key() -> (SigningKey, String) {
    let signing_key = SigningKey::from_bytes(&rand::random());
    let public_key = encode_ssh_ed25519_public_key(&signing_key.verifying_key());
    (signing_key, public_key)
}

#[cfg(test)]
pub fn verify_server_signature(
    identity: &ServerIdentity,
    challenge: &str,
    signature: &str,
) -> Result<()> {
    let signature = Signature::from_slice(&BASE64.decode(signature)?)?;
    identity
        .signing_key
        .verifying_key()
        .verify(challenge.as_bytes(), &signature)?;
    Ok(())
}
