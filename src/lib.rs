use crate::error::NpsError;
use axum::Router;
use std::sync::Arc;

pub mod error;
mod handlers;
mod payloads;
pub mod routes;
mod segment;

pub type AppResult<T> = Result<T, NpsError>;

pub struct AppState {}

pub fn app(state: Arc<AppState>) -> Router {
    routes::routes().with_state(state)
}
