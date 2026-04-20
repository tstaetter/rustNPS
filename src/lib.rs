pub mod db;
pub mod error;
pub mod handlers;
mod payloads;
pub mod routes;
pub mod segment;

// Re-export useful types for testing
pub use crate::db::NpsEntry;
pub use crate::handlers::create;
pub use crate::handlers::dismiss as dismiss_handler;
pub use crate::handlers::index;
pub use crate::payloads::{IndexQuery, NpsCreatePayload, NpsDismissPayload};

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

// Re-export index handler for testing
pub use index as index_handler;
