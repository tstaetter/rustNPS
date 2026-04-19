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

impl NpsCreatePayload {
    pub fn new() -> Self {
        NpsCreatePayload::default()
    }
}
