use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum NpsError {
    #[error("Internal Server Error")]
    ServerInternal,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),
}

impl IntoResponse for NpsError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            NpsError::ServerInternal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            NpsError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            NpsError::Mongo(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
