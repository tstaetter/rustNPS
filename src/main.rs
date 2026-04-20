use mongodb::Client;
use rust_nps::{app, AppResult, AppState};
use std::sync::Arc;
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();
    // Init tracing, write logs to STDOUT
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(std::io::stdout),
        )
        .init();

    // Database setup
    let mongo_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = std::env::var("MONGODB_DB").unwrap_or_else(|_| "rust_nps".to_string());
    let client = Client::with_uri_str(&mongo_uri).await?;
    let db = client.database(&mongo_db);
    let app_state = Arc::new(AppState { db });
    let app = app(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
