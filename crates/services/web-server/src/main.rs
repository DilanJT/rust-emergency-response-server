//! Dubai Healthcare Emergency Response System - Web Server
//! Main entry point for the Axum web server

use anyhow::Result;
use tracing_subscriber;

mod server;
mod web;
mod extractors;
mod responses;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("Starting Dubai Healthcare Emergency Response System");

    // Start the server
    server::start().await?;

    Ok(())
}
