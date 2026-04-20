pub mod db;
pub mod error;
mod handlers;
mod payloads;
pub mod routes;
pub mod segment;

// Re-export useful types for testing
pub use crate::handlers::create;
pub use crate::payloads::NpsCreatePayload;

use crate::error::NpsError;
use axum::Router;
pub use handlers::*;
use std::sync::Arc;

pub type AppResult<T> = Result<T, NpsError>;

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Database,
}

pub fn app(state: Arc<AppState>) -> Router {
    routes::routes().with_state(state)
}
