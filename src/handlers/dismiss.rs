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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dismiss_handler_empty_collection() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "User".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_dismissed_false() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Studio".to_string(),
            dismissed: false,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        // Dismissed false should still be CREATED
        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_string_segment() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Professional".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_invalid_segment() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Invalid_Segment".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let response = result.into_response();

        // Invalid segment should default to User, response should be CREATED
        assert_eq!(response.status(), StatusCode::CREATED);
    }

}
