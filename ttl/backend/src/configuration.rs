use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    pub server_signing_private_key_path: PathBuf,
    pub server_signing_public_key_path: PathBuf,
    pub challenge_expiry_minutes: i64,
    pub default_member_expiry_days: i64,
    #[allow(dead_code)]
    pub ttl_api_url: String,
    pub pocketbase_url: String,
    pub pocketbase_token: String,
    #[serde(default)]
    pub discord_admin_webhook_url: Option<String>,
    #[serde(default)]
    pub discord_access_webhook_url: Option<String>,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    pub tls_certificate_path: PathBuf,
    pub tls_private_key_path: PathBuf,
}

impl Configuration {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read configuration file {}", path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("invalid configuration file {}", path.display()))
    }

    pub fn path_from_args<I, S>(args: I) -> Result<PathBuf>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut args = args.into_iter().map(Into::into);
        let _program = args.next();
        let mut path = PathBuf::from("configuration.json");

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "-c" | "--config" => {
                    let value = args.next().context("missing path after -c/--config")?;
                    if value.trim().is_empty() {
                        anyhow::bail!("configuration path must not be empty");
                    }
                    path = PathBuf::from(value);
                }
                "-h" | "--help" => {
                    anyhow::bail!(
                        "usage: ttl-api [-c|--config <path>]\n\
                         default configuration path: ./configuration.json"
                    );
                }
                unknown => anyhow::bail!("unknown command-line argument '{unknown}'"),
            }
        }

        Ok(path)
    }
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}

fn default_bind_address() -> String {
    "0.0.0.0:3000".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_default_path_without_arguments() {
        assert_eq!(
            Configuration::path_from_args(["ttl-api"]).unwrap(),
            PathBuf::from("configuration.json")
        );
    }

    #[test]
    fn accepts_short_and_long_config_options() {
        assert_eq!(
            Configuration::path_from_args(["ttl-api", "-c", "custom.json"]).unwrap(),
            PathBuf::from("custom.json")
        );
        assert_eq!(
            Configuration::path_from_args(["ttl-api", "--config", "custom.json"]).unwrap(),
            PathBuf::from("custom.json")
        );
    }

    #[test]
    fn rejects_missing_config_path() {
        let error = Configuration::path_from_args(["ttl-api", "-c"]).unwrap_err();
        assert!(error.to_string().contains("missing path"));
    }

    #[test]
    fn defaults_optional_runtime_settings() {
        let configuration: Configuration = serde_json::from_str(
            r#"{
                "server_signing_private_key_path": "private",
                "server_signing_public_key_path": "public",
                "challenge_expiry_minutes": 5,
                "default_member_expiry_days": 365,
                "ttl_api_url": "http://localhost",
                "pocketbase_url": "",
                "pocketbase_token": "",
                "tls_certificate_path": "certificate.pem",
                "tls_private_key_path": "private-key.pem"
            }"#,
        )
        .unwrap();

        assert_eq!(configuration.data_dir, PathBuf::from("data"));
        assert_eq!(configuration.bind_address, "0.0.0.0:3000");
        assert_eq!(configuration.discord_admin_webhook_url, None);
        assert_eq!(configuration.discord_access_webhook_url, None);
    }

    #[test]
    fn accepts_discord_webhook_configuration() {
        let configuration: Configuration = serde_json::from_str(
            r#"{
                "server_signing_private_key_path": "private",
                "server_signing_public_key_path": "public",
                "challenge_expiry_minutes": 5,
                "default_member_expiry_days": 365,
                "ttl_api_url": "http://localhost",
                "pocketbase_url": "",
                "pocketbase_token": "",
                "tls_certificate_path": "certificate.pem",
                "tls_private_key_path": "private-key.pem",
                "discord_admin_webhook_url": "https://discord.example.com/admin",
                "discord_access_webhook_url": "https://discord.example.com/access"
            }"#,
        )
        .unwrap();

        assert_eq!(
            configuration.discord_admin_webhook_url.as_deref(),
            Some("https://discord.example.com/admin")
        );
        assert_eq!(
            configuration.discord_access_webhook_url.as_deref(),
            Some("https://discord.example.com/access")
        );
    }
}
