use anyhow::Result;
use tokio::net::TcpListener;
mod configuration;
mod crypto;
mod door_lock;
mod models;
mod persistence;
mod state;
mod web;

use crate::{
    configuration::Configuration, crypto::ServerIdentity, persistence::Repositories,
    state::AppState, web::app_router,
};

#[tokio::main]
async fn main() -> Result<()> {
    let configuration_path = Configuration::path_from_args(std::env::args())?;
    let configuration = Configuration::load(&configuration_path)?;
    let server_identity = ServerIdentity::load_from_files(
        &configuration.server_signing_private_key_path,
        &configuration.server_signing_public_key_path,
    )?;
    let repositories = Repositories::from_configuration(&configuration)?;
    let state = AppState::new_with_configuration(server_identity, repositories, &configuration)?;
    let app = app_router(state);

    let listener = TcpListener::bind(&configuration.bind_address).await?;
    println!("TTL API listening on http://{}", configuration.bind_address);
    axum::serve(listener, app).await?;
    Ok(())
}
