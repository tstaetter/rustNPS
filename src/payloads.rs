use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NpsCreatePayload {
    pub user: ObjectId,
    pub segment: String,
    pub score: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NpsDismissPayload {
    pub user: ObjectId,
    pub segment: String,
    pub dismissed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpsStats {
    pub total: u64,
    pub promoters: u64,
    pub passives: u64,
    pub detractors: u64,
    pub nps: i32,
    pub promoter_pct: f64,
    pub passive_pct: f64,
    pub detractor_pct: f64,
    pub average: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendItem {
    pub label: String,
    pub overall: i32,
    pub by_segment: std::collections::HashMap<String, i32>,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NpsDashboardResponse {
    pub period_days: i32,
    pub overall: NpsStats,
    pub segments: std::collections::HashMap<String, NpsStats>,
    pub trend: Vec<TrendItem>,
}

#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    pub period: Option<i32>,
}

impl NpsCreatePayload {
    pub fn new() -> Self {
        NpsCreatePayload::default()
    }
}
