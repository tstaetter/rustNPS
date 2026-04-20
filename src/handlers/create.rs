use axum::response::IntoResponse;

pub async fn create() -> impl IntoResponse {
    tracing::info!("Creating new notification");
}
