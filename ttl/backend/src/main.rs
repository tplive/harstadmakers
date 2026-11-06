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

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to install the Rustls ring crypto provider"))?;
    let configuration_path = Configuration::path_from_args(std::env::args())?;
    let configuration = Configuration::load(&configuration_path)?;
    let server_identity = ServerIdentity::load_from_files(
        &configuration.server_signing_private_key_path,
        &configuration.server_signing_public_key_path,
    )?;
    let repositories = Repositories::from_configuration(&configuration)?;
    let state = AppState::new_with_configuration(server_identity, repositories, &configuration)?;
    let app = app_router(state);

    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
        &configuration.tls_certificate_path,
        &configuration.tls_private_key_path,
    )
    .await?;
    println!(
        "TTL API listening on https://{}",
        configuration.bind_address
    );
    axum_server::bind_rustls(configuration.bind_address.parse()?, tls_config)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
