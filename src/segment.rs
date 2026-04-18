use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Segment {
    Promoters,
    Negative,
    Neutral,
}
