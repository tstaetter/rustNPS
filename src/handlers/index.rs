use super::stats::{build_stats, build_trend};
use crate::db::NpsEntry;
use crate::payloads::{IndexQuery, NpsDashboardResponse};
use crate::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use bson::{doc, DateTime};
use chrono::{Duration, Utc};
use mongodb::Collection;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn index(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IndexQuery>,
) -> impl IntoResponse {
    let period_days = query.period.unwrap_or(90);
    let from = Utc::now() - Duration::days(period_days as i64);
    let from_bson = DateTime::from_chrono(from);

    let collection: Collection<NpsEntry> = state.db.collection("nps_responses");

    // Get segments
    let segments_names: Vec<String> = collection
        .distinct("segment", doc! { "created_at": { "$gte": from_bson } })
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|b| b.as_str().map(|s| s.to_string()))
        .collect();

    // Overall stats
    let base_filter = doc! { "created_at": { "$gte": from_bson } };
    let overall = build_stats(&collection, base_filter.clone()).await;

    // Segment stats
    let mut segments = HashMap::new();
    for segment in segments_names {
        let mut filter = base_filter.clone();
        filter.insert("segment", &segment);
        segments.insert(segment, build_stats(&collection, filter).await);
    }

    // Trend
    let trend = build_trend(&collection).await;

    Json(NpsDashboardResponse {
        period_days,
        overall,
        segments,
        trend,
    })
}
