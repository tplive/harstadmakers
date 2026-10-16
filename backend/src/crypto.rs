use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use rand::{rngs::OsRng, Rng};
use rsa::BigUint;
use rsa::{
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, LineEnding},
    Oaep, RsaPrivateKey, RsaPublicKey,
};
use sha2::Sha256;
use ssh_key::{private::KeypairData, PrivateKey};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::models::Member;

#[derive(Clone)]
pub struct ServerIdentity {
    public_key_openssh: String,
    signing_key: SigningKey,
}

impl ServerIdentity {
    pub fn load_from_default_ssh_dir() -> Result<Self> {
        let home_dir = std::env::var("HOME").context("HOME is not set")?;
        let ssh_dir = PathBuf::from(home_dir).join(".ssh");
        Self::load_from_ssh_dir(&ssh_dir)
    }

    pub fn load_from_ssh_dir(ssh_dir: &Path) -> Result<Self> {
        let private_key_path = ssh_dir.join("id_ed25519");
        let public_key_path = if ssh_dir.join("id_ed25519.pub").exists() {
            ssh_dir.join("id_ed25519.pub")
        } else {
            ssh_dir.join("if_ed25519.pub")
        };

        Self::load_from_files(&private_key_path, &public_key_path)
    }

    pub fn load_from_files(private_key_path: &Path, public_key_path: &Path) -> Result<Self> {
        if !private_key_path.exists() {
            anyhow::bail!(
                "missing required server private key at {}",
                private_key_path.display()
            );
        }

        if !public_key_path.exists() {
            anyhow::bail!(
                "missing required server public key at {}",
                public_key_path.display()
            );
        }

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

        let signature = self.signing_key.sign(challenge.as_bytes());
        Ok(BASE64.encode(signature.to_bytes()))
    }
}

pub fn create_demo_member() -> Member {
    let (private_key, public_key) =
        generate_demo_keypair().expect("failed to generate demo keypair");
    let public_key_pem = public_key
        .to_pkcs1_pem(LineEnding::LF)
        .expect("failed to serialize public key")
        .to_string();

    println!("Demo member ID: demo-member");
    println!("Demo member private key (for mobile device testing only):");
    println!(
        "{}",
        private_key
            .to_pkcs1_pem(LineEnding::LF)
            .expect("failed to serialize private key")
            .to_string()
    );
    println!("Demo member public key registered in the app:");
    println!("{public_key_pem}");

    Member {
        id: "demo-member".to_string(),
        name: "Demo Member".to_string(),
        email: "demo@harstadmakers.no".to_string(),
        public_key_pem,
        active: true,
        is_admin: true,
        created_at: Utc::now().to_rfc3339(),
        last_used: None,
    }
}

pub fn generate_demo_keypair() -> Result<(RsaPrivateKey, RsaPublicKey)> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = RsaPublicKey::from(&private_key);
    Ok((private_key, public_key))
}

pub fn normalize_public_key_pem(
    public_key_pem: Option<String>,
    public_key_jwk: Option<serde_json::Value>,
) -> Result<String> {
    if let Some(pem) = public_key_pem {
        let public_key =
            RsaPublicKey::from_pkcs1_pem(&pem).context("invalid PKCS#1 RSA public key PEM")?;
        return Ok(public_key
            .to_pkcs1_pem(LineEnding::LF)
            .context("failed to re-encode public key")?
            .to_string());
    }

    if let Some(jwk) = public_key_jwk {
        let modulus = jwk
            .get("n")
            .and_then(|v| v.as_str())
            .context("missing JWK modulus field 'n'")?;
        let exponent = jwk
            .get("e")
            .and_then(|v| v.as_str())
            .context("missing JWK exponent field 'e'")?;

        let modulus_bytes = base64url_decode(modulus)?;
        let exponent_bytes = base64url_decode(exponent)?;

        let modulus_bigint = BigUint::from_bytes_be(&modulus_bytes);
        let exponent_bigint = BigUint::from_bytes_be(&exponent_bytes);

        let public_key = RsaPublicKey::new(modulus_bigint, exponent_bigint)
            .context("invalid RSA public key values in JWK")?;

        return Ok(public_key
            .to_pkcs1_pem(LineEnding::LF)
            .context("failed to encode JWK public key as PEM")?
            .to_string());
    }

    anyhow::bail!("missing RSA public key value")
}

pub fn random_nonce() -> String {
    let bytes: Vec<u8> = (0..32)
        .map(|_| rand::thread_rng().gen_range(b'a'..=b'z'))
        .collect();
    String::from_utf8(bytes).expect("nonce generation should never produce invalid UTF-8")
}

pub fn encrypt_with_public_key(public_key_pem: &str, message: &str) -> Result<String> {
    let public_key = RsaPublicKey::from_pkcs1_pem(public_key_pem)
        .context("failed to parse registered member key")?;
    let mut rng = OsRng;
    let encrypted = public_key.encrypt(&mut rng, Oaep::new::<Sha256>(), message.as_bytes())?;
    Ok(BASE64.encode(encrypted))
}

fn base64url_decode(value: &str) -> Result<Vec<u8>> {
    let mut normalized = value.replace('-', "+").replace('_', "/");
    while normalized.len() % 4 != 0 {
        normalized.push('=');
    }
    Ok(BASE64.decode(&normalized)?)
}

#[cfg(test)]
pub fn generate_test_server_identity() -> ServerIdentity {
    let secret_key_bytes: [u8; 32] = rand::random();
    let signing_key = SigningKey::from_bytes(&secret_key_bytes);
    let public_key_openssh = format!(
        "ssh-ed25519 {} test-server@harstadmakers",
        encode_ssh_ed25519_public_key(&signing_key.verifying_key())
    );

    ServerIdentity {
        public_key_openssh,
        signing_key,
    }
}

#[cfg(test)]
pub fn verify_server_signature(
    identity: &ServerIdentity,
    challenge: &str,
    signature_b64: &str,
) -> Result<()> {
    let signature_bytes = BASE64.decode(signature_b64)?;
    let signature = Signature::from_slice(&signature_bytes)?;
    identity
        .signing_key
        .verifying_key()
        .verify(challenge.as_bytes(), &signature)?;
    Ok(())
}

#[cfg(test)]
fn encode_ssh_ed25519_public_key(verifying_key: &VerifyingKey) -> String {
    let algorithm = b"ssh-ed25519";
    let key_bytes = verifying_key.to_bytes();
    let mut wire = Vec::with_capacity(4 + algorithm.len() + 4 + key_bytes.len());
    wire.extend_from_slice(&(algorithm.len() as u32).to_be_bytes());
    wire.extend_from_slice(algorithm);
    wire.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());
    wire.extend_from_slice(&key_bytes);
    BASE64.encode(wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsa_challenge_round_trip_works() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let message = "door-access-123";

        let encrypted = public_key
            .encrypt(&mut rng, Oaep::new::<Sha256>(), message.as_bytes())
            .unwrap();
        let decrypted = private_key
            .decrypt(Oaep::new::<Sha256>(), &encrypted)
            .unwrap();

        assert_eq!(String::from_utf8(decrypted).unwrap(), message);
    }

    #[test]
    fn server_identity_signs_challenges() {
        let identity = generate_test_server_identity();
        let signature = identity.sign_challenge("verify-me").unwrap();

        assert!(verify_server_signature(&identity, "verify-me", &signature).is_ok());
    }
}
