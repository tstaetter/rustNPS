use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NpsCreatePayload {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user: ObjectId,
    pub segment: String,
    pub score: i32,
    pub comment: Option<String>,
    pub dismissed: Option<bool>,
}

impl NpsCreatePayload {
    pub fn new() -> Self {
        NpsCreatePayload::default()
    }
}
