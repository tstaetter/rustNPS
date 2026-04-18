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
    let app_state = Arc::new(AppState {});
    let app = app(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
