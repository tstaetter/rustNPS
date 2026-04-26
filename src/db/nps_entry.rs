use crate::segment::Segment;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NpsEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user: ObjectId,
    pub segment: Segment,
    pub score: i32,
    pub comment: Option<String>,
    pub dismissed: Option<bool>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::payloads::NpsCreatePayload> for NpsEntry {
    fn from(entry: crate::payloads::NpsCreatePayload) -> Self {
        Self {
            id: None,
            user: entry.user,
            segment: Segment::from(entry.segment),
            score: entry.score,
            comment: entry.comment,
            dismissed: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
