use crate::AppState;
use crate::db::NpsEntry;
use crate::payloads::NpsDismissPayload;
use crate::segment::Segment;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bson::doc;
use serde_json::json;
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

    let collection: mongodb::Collection<NpsEntry> = state.db.collection("nps_entries");
    let segment = Segment::from(payload.segment);
    let segment_str = match segment {
        Segment::User => "User",
        Segment::Studio => "Studio",
        Segment::Professional => "Professional",
    };
    let filter = doc! {
        "user": payload.user,
        "segment": segment_str
    };
    let update = doc! {
        "$set": {
            "dismissed": payload.dismissed,
            "updated_at": chrono::Utc::now()
        }
    };

    match collection.update_one(filter, update).await {
        Ok(result) => {
            if result.matched_count == 0 {
                tracing::info!("No matching entry found for dismiss");
                let response = json!({ "error": "Not found" });
                (StatusCode::NOT_FOUND, Json(response)).into_response()
            } else {
                tracing::info!("Entry dismissed successfully");
                let response = json!({ "data": { "message": "Updated" } });
                (StatusCode::OK, Json(response)).into_response()
            }
        }
        Err(e) => {
            tracing::error!("Error dismissing entry: {:?}", e);
            let response = json!({ "error": e.to_string() });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::NpsEntry;

    #[tokio::test]
    async fn test_dismiss_handler_empty_collection() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "User".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_dismissed_false() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Studio".to_string(),
            dismissed: false,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        // No matching entry exists, should return NOT_FOUND
        assert_eq!(status.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_string_segment() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Professional".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_dismiss_handler_with_invalid_segment() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: "Invalid_Segment".to_string(),
            dismissed: true,
        };

        let result = dismiss(State(app_state), Json(payload)).await;
        let response = result.into_response();

        // Invalid segment should be rejected by validation
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
