//! Integration tests for the create handler
//!
//! These tests verify that the NPS create handler works correctly
//! by testing various score values, segments, and payload configurations.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use rust_nps::{AppState, db::NpsEntry, create, NpsCreatePayload};
use std::sync::Arc;
use bson::oid::ObjectId;

#[tokio::test]
async fn test_create_handler_returns_created_status() {
    // Setup: Create test database
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    // Test with different score values
    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 9,
        comment: Some("Great product!".to_string()),
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_promoter_score() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Studio".to_string(),
        score: 10,
        comment: Some("Perfect!".to_string()),
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_detractor_score() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Professional".to_string(),
        score: 3,
        comment: Some("Very bad".to_string()),
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_passive_score() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 7,
        comment: Some("Neutral".to_string()),
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_without_comment() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Studio".to_string(),
        score: 8,
        comment: None,
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_valid_object_id() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 9,
        comment: None,
    };

    let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
    let (status, _) = response.into_response().into_parts();

    // Valid ObjectId should succeed
    assert_eq!(status.status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_all_segments() {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let _collection = db.collection::<NpsEntry>("nps_entries");

    let app_state = AppState { db: db.clone() };

    // Test all valid segments
    let segments = vec!["User", "Studio", "Professional"];
    for segment in segments {
        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: segment.to_string(),
            score: 8,
            comment: Some(format!("Test {} score", segment)),
        };

        let response = create::create(State(Arc::new(app_state.clone())), Json(payload)).await;
        let (status, _) = response.into_response().into_parts();

        assert_eq!(status.status, StatusCode::CREATED);
    }
}
