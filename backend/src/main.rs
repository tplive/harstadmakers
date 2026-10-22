use anyhow::Result;
use std::path::PathBuf;
use tokio::net::TcpListener;
mod crypto;
mod models;
mod persistence;
mod state;
mod web;

use crate::{crypto::ServerIdentity, persistence::FileRepositories, state::AppState, web::app_router};

#[tokio::main]
async fn main() -> Result<()> {
    let data_dir = PathBuf::from("data");
    let server_identity = ServerIdentity::load_from_default_ssh_dir()?;
    let repositories = FileRepositories::new(data_dir)?;
    let state = AppState::new(server_identity, repositories)?;
    let app = app_router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Harstad Makers backend listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
