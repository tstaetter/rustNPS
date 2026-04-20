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
        tracing::error!("Validation failed for entry: {:?}", e);
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

#[cfg(test)]
mod tests {
    use super::*;
    use bson::oid::ObjectId;

    #[tokio::test]
    async fn test_create_handler_empty_collection() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: "User".to_string(),
            score: 9,
            comment: Some("Test comment".to_string()),
        };

        let result = create(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_handler_with_promoter_score() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: "Studio".to_string(),
            score: 10,
            comment: None,
        };

        let result = create(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_handler_with_detractor_score() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: "Professional".to_string(),
            score: 4,
            comment: Some("Terrible experience".to_string()),
        };

        let result = create(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_handler_with_passive_score() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        // Test with score 8 (passive)
        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: "Studio".to_string(),
            score: 8,
            comment: Some("Good but not great".to_string()),
        };

        let result = create(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_handler_with_detractor_score_6() {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps")
            .await
            .unwrap();

        let db = client.database("test_nps");
        let _collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

        let app_state = Arc::new(AppState { db: db.clone() });

        // Test with score 6 (detractor)
        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: "Professional".to_string(),
            score: 6,
            comment: Some("Disappointing".to_string()),
        };

        let result = create(State(app_state), Json(payload)).await;
        let (status, _) = result.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }
}
