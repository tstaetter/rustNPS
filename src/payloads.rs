use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Clone, Default, Validate)]
pub struct NpsCreatePayload {
    pub user: ObjectId,
    #[validate(custom(function = "validate_segment"))]
    pub segment: String,
    #[validate(range(min = 0, max = 10))]
    pub score: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Validate)]
pub struct NpsDismissPayload {
    pub user: ObjectId,
    #[validate(custom(function = "validate_segment"))]
    pub segment: String,
    pub dismissed: bool,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct TrendItem {
    pub label: String,
    pub overall: i32,
    pub by_segment: std::collections::HashMap<String, i32>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct NpsDashboardResponse {
    pub period_days: i32,
    pub overall: NpsStats,
    pub segments: std::collections::HashMap<String, NpsStats>,
    pub trend: Vec<TrendItem>,
}

#[derive(Debug, Deserialize, Default, Validate)]
pub struct IndexQuery {
    #[validate(range(min = 1, max = 730))]
    pub period: Option<i32>,
}

fn validate_segment(segment: &str) -> Result<(), ValidationError> {
    match segment {
        "User" | "Studio" | "Professional" => Ok(()),
        _ => Err(
            ValidationError::new("invalid_segment").with_message(std::borrow::Cow::Owned(format!(
                "Invalid segment: '{}'. Must be one of: User, Studio, Professional",
                segment
            ))),
        ),
    }
}
