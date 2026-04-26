//! Integration tests for the dismiss handler
//!
//! These tests verify that the NPS dismiss handler works correctly
//! by testing various dismiss scenarios and response structures.

use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::Utc;
use rust_nps::{AppState, NpsDismissPayload, db::NpsEntry, segment::Segment};
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
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert a test entry with a known user ID
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();
    let entry = NpsEntry {
        id: None,
        user: user_id.clone(),
        segment: Segment::User,
        score: 9,
        comment: Some("Test comment".to_string()),
        dismissed: None,
        created_at: now,
        updated_at: now,
    };
    collection.insert_one(entry).await.unwrap();

    // Dismiss with matching user and segment
    let payload = NpsDismissPayload {
        user: user_id,
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["msg"], "Updated");
}

#[tokio::test]
async fn test_dismiss_handler_not_found_when_no_matching_entry() {
    let (server, _) = setup_test_server().await;

    // Dismiss with no matching entry should return 404
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::NOT_FOUND);

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["msg"], "Not found");
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
    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dismiss_handler_dismissed_false() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert a test entry with a known user ID
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();
    let entry = NpsEntry {
        id: None,
        user: user_id.clone(),
        segment: Segment::Studio,
        score: 8,
        comment: None,
        dismissed: None,
        created_at: now,
        updated_at: now,
    };
    collection.insert_one(entry).await.unwrap();

    // Test dismiss handler with dismissed=false on a matching entry
    let payload = NpsDismissPayload {
        user: user_id,
        segment: "Studio".to_string(),
        dismissed: false,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_dismiss_handler_all_segments() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert entries for all segments with the same user
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();

    for segment in [Segment::User, Segment::Studio, Segment::Professional] {
        let entry = NpsEntry {
            id: None,
            user: user_id.clone(),
            segment: segment.clone(),
            score: 9,
            comment: None,
            dismissed: None,
            created_at: now,
            updated_at: now,
        };
        collection.insert_one(entry).await.unwrap();
    }

    // Test dismiss handler with all segments
    let segments = vec!["User", "Studio", "Professional"];

    for segment in segments {
        let payload = NpsDismissPayload {
            user: user_id.clone(),
            segment: segment.to_string(),
            dismissed: true,
        };

        let response = server.delete("/v1/nps/dismiss").json(&payload).await;
        response.assert_status(StatusCode::OK);
    }
}

#[tokio::test]
async fn test_dismiss_handler_invalid_segment_rejected() {
    let (server, _db) = setup_test_server().await;

    // Dismiss with invalid segment should be rejected by validation
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "Invalid_Segment".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    // Invalid segment is now rejected by validation with 422
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_dismiss_handler_response_body() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert a test entry
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();
    let entry = NpsEntry {
        id: None,
        user: user_id.clone(),
        segment: Segment::User,
        score: 9,
        comment: Some("Test comment".to_string()),
        dismissed: None,
        created_at: now,
        updated_at: now,
    };
    collection.insert_one(entry).await.unwrap();

    // Test dismiss handler with matching entry
    let payload = NpsDismissPayload {
        user: user_id,
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Verify response contains expected fields
    assert!(json.get("msg").is_some());
    assert_eq!(json["msg"], "Updated");
}

#[tokio::test]
async fn test_dismiss_handler_multiple_entries_same_user() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Create entries for the same user across different segments
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();

    for segment in [Segment::User, Segment::Studio] {
        let entry = NpsEntry {
            id: None,
            user: user_id.clone(),
            segment: segment,
            score: 9,
            comment: None,
            dismissed: None,
            created_at: now,
            updated_at: now,
        };
        collection.insert_one(entry).await.unwrap();
    }

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
        response.assert_status(StatusCode::OK);
    }
}

#[tokio::test]
async fn test_dismiss_handler_different_periods() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert test entries with a known user ID
    let user_id = bson::oid::ObjectId::new();
    let now = Utc::now();
    for i in 0..5 {
        let entry = NpsEntry {
            id: None,
            user: user_id.clone(),
            segment: Segment::User,
            score: 9,
            comment: Some(format!("Test comment {}", i)),
            dismissed: None,
            created_at: now - chrono::Duration::days(i as i64),
            updated_at: now - chrono::Duration::days(i as i64),
        };
        collection.insert_one(entry).await.unwrap();
    }

    // Dismiss with matching user and segment
    let payload = NpsDismissPayload {
        user: user_id,
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_dismiss_handler_wrong_user_returns_not_found() {
    let (server, db) = setup_test_server().await;
    let collection: mongodb::Collection<NpsEntry> = db.collection("nps_entries");

    // Insert a test entry for one user
    let entry_user_id = bson::oid::ObjectId::new();
    let now = Utc::now();
    let entry = NpsEntry {
        id: None,
        user: entry_user_id,
        segment: Segment::User,
        score: 9,
        comment: None,
        dismissed: None,
        created_at: now,
        updated_at: now,
    };
    collection.insert_one(entry).await.unwrap();

    // Try to dismiss with a different user ID
    let payload = NpsDismissPayload {
        user: bson::oid::ObjectId::new(),
        segment: "User".to_string(),
        dismissed: true,
    };

    let response = server.delete("/v1/nps/dismiss").json(&payload).await;
    response.assert_status(StatusCode::NOT_FOUND);
}
