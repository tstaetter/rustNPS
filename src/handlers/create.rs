use crate::db::NpsEntry;
use crate::payloads::NpsCreatePayload;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use bson::doc;
use mongodb::Collection;
use std::sync::Arc;
use validator::Validate;

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NpsCreatePayload>,
) -> impl IntoResponse {
    tracing::info!("Creating new notification");

    if let Err(e) = payload.validate() {
        tracing::error!("Validation failed for booking: {:?}", e);
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(e)).into_response();
    }

    let entry = NpsEntry::from(payload);
    let collection: Collection<NpsEntry> = state.db.collection("nps_entries");

    match collection.insert_one(entry.clone()).await {
        Ok(result) => {
            let mut created_entry = entry;

            created_entry.id = result.inserted_id.as_object_id();
            tracing::info!("Entry created with id: {:?}", created_entry.id);

            let response = doc! { "msg": "Created" };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Error creating entry: {:?}", e);

            let response = doc! { "msg": e.to_string() };

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
