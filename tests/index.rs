//! Integration tests for the index handler
//!
//! These tests verify that the NPS index handler works correctly
//! by testing various query parameters and response structures.

use axum::http::StatusCode;
use axum_test::TestServer;
use bson::doc;
use bson::oid::ObjectId;
use chrono::{Duration, Utc};
use mongodb::Client;
use rust_nps::AppState;
use std::sync::Arc;

async fn setup_test_server(db_name: &str) -> (TestServer, mongodb::Database) {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();

    let db = client.database(db_name);
    let app_state = Arc::new(AppState { db: db.clone() });

    // Clear any existing data
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");
    collection.delete_many(doc! {}).await.unwrap();

    let app = rust_nps::app(app_state);
    let server = TestServer::new(app);
    (server, db)
}

#[tokio::test]
async fn test_index_handler_returns_dashboard_with_default_period() {
    let (server, db) = setup_test_server("test_nps_default").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Insert sample data for the past 90 days
    let now = Utc::now();
    for day in 0..10 {
        let date = (now - Duration::days(day as i64))
            .naive_local()
            .and_local_timezone(Utc)
            .unwrap();

        let entry = doc! {
            "user": ObjectId::new(),
            "segment": "User",
            "score": if day % 2 == 0 { 10 } else { 7 },
            "comment": format!("Test comment day {}", day),
            "created_at": date,
            "updated_at": date,
        };
        collection.insert_one(entry).await.unwrap();
    }

    // Test with default period (90 days)
    let response = server.get("/v1/nps").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Verify response structure
    assert!(json.get("overall").is_some());
    assert!(json.get("segments").is_some());
    assert!(json.get("trend").is_some());
    assert_eq!(json["period_days"], 90);

    // Verify overall stats have data
    assert!(json["overall"]["total"].as_u64().unwrap_or(0) > 0);
}

#[tokio::test]
async fn test_index_handler_returns_dashboard_with_custom_period() {
    let (server, db) = setup_test_server("test_nps_custom").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Insert sample data for the past 30 days
    let now = Utc::now();
    for day in 0..10 {
        let date = (now - Duration::days(day as i64))
            .naive_local()
            .and_local_timezone(Utc)
            .unwrap();

        let entry = doc! {
            "user": ObjectId::new(),
            "segment": "Studio",
            "score": 9,
            "comment": format!("Test comment day {}", day),
            "created_at": date,
            "updated_at": date,
        };
        collection.insert_one(entry).await.unwrap();
    }

    // Test with custom period (30 days)
    let response = server.get("/v1/nps").add_query_param("period", 30).await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Verify period_days is set correctly
    assert_eq!(json["period_days"], 30);
}

#[tokio::test]
async fn test_index_handler_with_multiple_segments() {
    let (server, db) = setup_test_server("test_nps_segments").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Insert sample data with multiple segments
    let now = Utc::now();
    let segments = vec!["User", "Studio", "Professional"];
    for segment_name in &segments {
        for day in 0..5 {
            let date = (now - Duration::days(day as i64))
                .naive_local()
                .and_local_timezone(Utc)
                .unwrap();

            let entry = doc! {
                "user": ObjectId::new(),
                "segment": segment_name.to_string(),
                "score": if day % 3 == 0 {
                    10
                } else if day % 3 == 1 {
                    7
                } else {
                    4
                },
                "comment": format!("Test comment day {} segment {}", day, segment_name),
                "created_at": date,
                "updated_at": date,
            };
            collection.insert_one(entry).await.unwrap();
        }
    }

    // Test with default period
    let response = server.get("/v1/nps").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Verify response has multiple segments
    assert!(json["segments"].as_object().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_index_handler_with_trend_data() {
    let (server, db) = setup_test_server("test_nps_trend").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Insert sample data for multiple months to test trend
    let now = Utc::now();
    for month_offset in 0..5 {
        let start_date = now - Duration::days((month_offset * 30) as i64);
        for day in 0..5 {
            let date = start_date + Duration::days(day as i64);
            let date_tz = date.naive_local().and_local_timezone(Utc).unwrap();

            let entry = doc! {
                "user": ObjectId::new(),
                "segment": "User",
                "score": 8,
                "comment": format!("Test comment month {} day {}", month_offset, day),
                "created_at": date_tz,
                "updated_at": date_tz,
            };
            collection.insert_one(entry).await.unwrap();
        }
    }

    // Test with 90 day period (should include at least some trend data)
    let response = server.get("/v1/nps").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Verify trend array exists and has expected number of months
    let trend = json["trend"].as_array().unwrap();
    assert_eq!(trend.len(), 6); // Should return 6 months of trend data
}

#[tokio::test]
async fn test_index_handler_with_score_distribution() {
    let (server, db) = setup_test_server("test_nps_distribution").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Insert sample data with known score distribution
    let now = Utc::now();
    let promoters = [10, 10, 9, 9, 9];
    let passives = [8, 7, 8, 7, 8];
    let detractors = [6, 5, 4, 3, 2];

    for score_group in [&promoters[..], &passives[..], &detractors[..]] {
        for score in score_group {
            let entry = doc! {
                "user": ObjectId::new(),
                "segment": "User",
                "score": *score,
                "comment": "Test score distribution",
                "created_at": now,
                "updated_at": now,
            };
            collection.insert_one(entry).await.unwrap();
        }
    }

    // Test with 90 day period
    let response = server.get("/v1/nps").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    let overall = &json["overall"];

    // Verify score distribution
    assert_eq!(overall["promoters"].as_u64().unwrap_or(0), 5);
    assert_eq!(overall["passives"].as_u64().unwrap_or(0), 5);
    assert_eq!(overall["detractors"].as_u64().unwrap_or(0), 5);
    assert_eq!(overall["total"].as_u64().unwrap_or(0), 15);
}

#[tokio::test]
async fn test_index_handler_with_empty_trend_data() {
    let (server, db) = setup_test_server("test_nps_empty").await;
    let collection: mongodb::Collection<bson::Document> = db.collection("nps_responses");

    // Clear any existing data
    collection.delete_many(doc! {}).await.unwrap();

    // Test with empty collection
    let response = server.get("/v1/nps").await;
    response.assert_status(StatusCode::OK);

    let json = response.json::<serde_json::Value>();

    // Empty collection should return zero stats
    assert_eq!(json["overall"]["total"], 0);
    assert_eq!(json["overall"]["promoters"], 0);
    assert_eq!(json["overall"]["passives"], 0);
    assert_eq!(json["overall"]["detractors"], 0);
    assert_eq!(json["overall"]["nps"], 0);
    assert_eq!(json["overall"]["promoter_pct"], 0.0);
    assert_eq!(json["overall"]["passive_pct"], 0.0);
    assert_eq!(json["overall"]["detractor_pct"], 0.0);
    assert_eq!(json["overall"]["average"], 0.0);

    // Trend should be empty array or all zeros
    let trend = json["trend"].as_array().unwrap();
    for item in trend {
        assert_eq!(item["overall"].as_i64().unwrap_or(0), 0);
        assert_eq!(item["total"].as_u64().unwrap_or(0), 0);
    }
}
