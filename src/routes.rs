use crate::{handlers, AppState};
use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    let nps_routes = Router::new()
        .route("/nps", get(handlers::index))
        .route("/nps", post(handlers::create))
        .route("/nps/dismiss", delete(handlers::dismiss));

    Router::new().nest("/v1", nps_routes)
}
