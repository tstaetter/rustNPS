use crate::db::NpsEntry;
use crate::payloads::NpsDismissPayload;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use bson::doc;
use std::sync::Arc;
use validator::Validate;

pub async fn dismiss(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NpsDismissPayload>,
) -> impl IntoResponse {
    tracing::info!("Dismissing notification");

    if let Err(e) = payload.validate() {
        tracing::error!("Validation failed for dismissed entry: {:?}", e);
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(e)).into_response();
    }

    let entry = NpsEntry::from(payload);
    let collection = state.db.collection("nps_responses");

    match collection.insert_one(entry.clone()).await {
        Ok(result) => {
            let mut dismissed_entry = entry;

            dismissed_entry.id = result.inserted_id.as_object_id();
            tracing::info!("Entry dismissed created with id: {:?}", dismissed_entry.id);

            let response = doc! { "msg": "Created" };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Error creating dismissed entry: {:?}", e);

            let response = doc! { "msg": e.to_string() };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
