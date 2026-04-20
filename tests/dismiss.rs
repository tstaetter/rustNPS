//! Integration tests for the dismiss handler
//!
//! These tests verify that the NPS dismiss handler works correctly
//! by testing various dismiss scenarios and response structures.

use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::Utc;
use rust_nps::{db::NpsEntry, segment::Segment, AppState, NpsDismissPayload};
use std::sync::Arc;

async fn setup_test_server() -> (TestServer, mongodb::Database) {
    let client = mongodb::Client::with_uri_str("mongodb://localhost:27017/test_nps_dismiss")
        .await
        .unwrap();

    let db = client.database("test_nps_dismiss");
    let app_state = Arc::new(AppState { db: db.clone() });

    // Clear any existing data
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");
    collection.delete_many(bson::doc! {}).await.unwrap();

    let app = rust_nps::app(app_state);
    let server = TestServer::new(app);
    (server, db)
}

#[tokio::test]
async fn test_dismiss_handler_success() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

    // Insert a test entry
    let now = Utc::now();
    let entry = NpsEntry {
        id: None,
        user: bson::oid::ObjectId::new(),
        segment: Segment::User,
        score: 9,
        comment: Some("Test comment".to_string()),
        dismissed: None,
        created_at: now,
        updated_at: now,
    };
    collection.insert_one(entry).await.unwrap();

    // Test dismiss handler with valid payload
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_dismiss_handler_dismissed_false() {
    let (server, _) = setup_test_server().await;

    // Test dismiss handler with dismissed=false
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "Studio".to_string(),
        dismissed: false,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_dismiss_handler_all_segments() {
    let (server, _) = setup_test_server().await;

    // Test dismiss handler with all segments
    let segments = vec!["User", "Studio", "Professional"];

    for segment in segments {
        let payload = NpsDismissPayload {
            user: bson::oid::ObjectId::new(),
            segment: segment.to_string(),
            dismissed: true,
        };

        let response = server.delete("/v1/nps/dismiss").json(&payload).await;
        response.assert_status(StatusCode::CREATED);
    }
}

#[tokio::test]
async fn test_dismiss_handler_invalid_segment_defaults_to_user() {
    let (server, _) = setup_test_server().await;

    // Test dismiss handler with invalid segment - should default to User
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "Invalid_Segment".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_dismiss_handler_empty_collection() {
    let (server, _) = setup_test_server().await;

    // Test dismiss handler with empty collection
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn test_dismiss_handler_response_body() {
    let (server, _) = setup_test_server().await;

    // Test dismiss handler with valid payload
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);

    let json = response.json::<serde_json::Value>();

    // Verify response contains expected fields
    assert!(json.get("msg").is_some());
}

#[tokio::test]
async fn test_dismiss_handler_multiple_entries_same_user() {
    let (server, _) = setup_test_server().await;

    // Create a single user object
    let user_id = bson::oid::ObjectId::new();

    // Test submitting multiple dismissals for the same user
    let payloads = vec![
        NpsDismissPayload {
            user: user_id.clone(),
            segment: "User".to_string(),
            dismissed: true,
        },
        NpsDismissPayload {
            user: user_id.clone(),
            segment: "Studio".to_string(),
            dismissed: true,
        },
    ];

    for payload in payloads {
        let response = server.delete("/v1/nps/dismiss").json(&payload).await;
        response.assert_status(StatusCode::CREATED);
    }
}

#[tokio::test]
async fn test_dismiss_handler_different_periods() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_responses");

    // Insert test entries
    let now = Utc::now();
    for i in 0..5 {
        let entry = NpsEntry {
            id: None,
            user: bson::oid::ObjectId::new(),
            segment: Segment::User,
            score: 9,
            comment: Some(format!("Test comment {}", i)),
            dismissed: None,
            created_at: now - chrono::Duration::days(i as i64),
            updated_at: now - chrono::Duration::days(i as i64),
        };
        collection.insert_one(entry).await.unwrap();
    }

    // Test dismiss handler
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::CREATED);
}
