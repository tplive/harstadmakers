use anyhow::Result;
mod configuration;
mod crypto;
mod door_lock;
mod models;
mod notifications;
mod persistence;
mod state;
mod web;

use crate::{
    configuration::Configuration, crypto::ServerIdentity, persistence::Repositories,
    state::AppState, web::app_router,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ttl_api=info,tower_http=info".into()),
        )
        .with_target(false)
        .compact()
        .init();

    let configuration_path = Configuration::path_from_args(std::env::args())?;
    info!(path = %configuration_path.display(), "loading configuration");
    let configuration = Configuration::load(&configuration_path)?;
    info!(
        bind_address = %configuration.bind_address,
        tls_enabled = configuration.tls_enabled,
        gpio_enabled = configuration.gpio_enabled,
        "configuration loaded"
    );
    let server_identity = ServerIdentity::load_from_files(
        &configuration.server_signing_private_key_path,
        &configuration.server_signing_public_key_path,
    )?;
    let repositories = Repositories::from_configuration(&configuration)?;
    let state = AppState::new_with_configuration(server_identity, repositories, &configuration)?;
    let app = app_router(state);

    if configuration.tls_enabled {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| anyhow::anyhow!("failed to install the Rustls ring crypto provider"))?;
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            &configuration.tls_certificate_path,
            &configuration.tls_private_key_path,
        )
        .await?;
        info!(address = %configuration.bind_address, "TTL API listening over HTTPS");
        axum_server::bind_rustls(configuration.bind_address.parse()?, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        info!(address = %configuration.bind_address, "TTL API listening over HTTP");
        let listener = tokio::net::TcpListener::bind(&configuration.bind_address).await?;
        axum::serve(listener, app).await?;
    }
    info!("HTTP server stopped");
    Ok(())
}
