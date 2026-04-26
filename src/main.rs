use bson::doc;
use mongodb::Client;
use mongodb::IndexModel;
use rust_nps::{AppResult, AppState, NpsEntry, app};
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, prelude::*};

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let mongo_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = std::env::var("MONGODB_DB").unwrap_or_else(|_| "rust_nps".to_string());
    let client = Client::with_uri_str(&mongo_uri).await?;
    let db = client.database(&mongo_db);

    // Create indexes
    let collection = db.collection::<NpsEntry>("nps_entries");
    collection
        .create_index(IndexModel::builder().keys(doc! { "created_at": 1 }).build())
        .await?;
    collection
        .create_index(IndexModel::builder().keys(doc! { "segment": 1 }).build())
        .await?;
    collection
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user": 1, "segment": 1 })
                .build(),
        )
        .await?;

    let app_state = Arc::new(AppState { db });
    let app = app(app_state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
