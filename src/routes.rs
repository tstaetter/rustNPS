use crate::{create, dismiss, index, AppState};
use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    let nps_routes = Router::new()
        .route("/nps", get(index::index))
        .route("/nps", post(create::create))
        .route("/nps/dismiss", delete(dismiss::dismiss));

    Router::new().nest("/v1", nps_routes)
}
