use axum::response::IntoResponse;

pub async fn index() -> impl IntoResponse {
    tracing::info!("Getting notifications");
}

pub async fn create() -> impl IntoResponse {
    tracing::info!("Creating new notification");
}

pub async fn dismiss() -> impl IntoResponse {
    tracing::info!("Dismissing notification");
}
