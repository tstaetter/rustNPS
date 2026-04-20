use axum::response::IntoResponse;

pub async fn dismiss() -> impl IntoResponse {
    tracing::info!("Dismissing notification");
}
