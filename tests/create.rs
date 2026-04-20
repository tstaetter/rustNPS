//! Integration tests for the create handler
//!
//! These tests verify that the NPS create handler works correctly
//! by testing various score values, segments, and payload configurations.

use axum::http::StatusCode;
use axum_test::TestServer;
use bson::oid::ObjectId;
use rust_nps::{AppState, NpsCreatePayload};
use std::sync::Arc;

async fn setup_test_server() -> TestServer {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_int")
        .await
        .unwrap();

    let db = client.database("test_nps_int");
    let app_state = Arc::new(AppState { db: db.clone() });

    // Clear any existing data
    let collection = db.collection::<bson::Document>("nps_entries");
    collection.delete_many(bson::doc! {}).await.unwrap();

    let app = rust_nps::app(app_state);
    TestServer::new(app)
}

#[tokio::test]
async fn test_create_handler_returns_created_status() {
    let server = setup_test_server().await;

    // Test with different score values
    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 9,
        comment: Some("Great product!".to_string()),
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_promoter_score() {
    let server = setup_test_server().await;

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Studio".to_string(),
        score: 10,
        comment: Some("Perfect!".to_string()),
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_detractor_score() {
    let server = setup_test_server().await;

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Professional".to_string(),
        score: 3,
        comment: Some("Very bad".to_string()),
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_passive_score() {
    let server = setup_test_server().await;

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 7,
        comment: Some("Neutral".to_string()),
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_without_comment() {
    let server = setup_test_server().await;

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "Studio".to_string(),
        score: 8,
        comment: None,
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_valid_object_id() {
    let server = setup_test_server().await;

    let payload = NpsCreatePayload {
        user: ObjectId::new(),
        segment: "User".to_string(),
        score: 9,
        comment: None,
    };

    let response = server.post("/v1/nps").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_handler_with_all_segments() {
    let server = setup_test_server().await;

    // Test all valid segments
    let segments = vec!["User", "Studio", "Professional"];
    for segment in segments {
        let payload = NpsCreatePayload {
            user: ObjectId::new(),
            segment: segment.to_string(),
            score: 8,
            comment: Some(format!("Test {} score", segment)),
        };

        let response = server.post("/v1/nps").json(&payload).await;
        response.assert_status(StatusCode::CREATED);
    }
}
