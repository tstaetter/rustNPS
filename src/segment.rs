use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum Segment {
    #[default]
    User,
    Studio,
    Professional,
}

impl From<String> for Segment {
    fn from(segment: String) -> Self {
        match segment.as_str() {
            "User" => Segment::User,
            "Studio" => Segment::Studio,
            "Professional" => Segment::Professional,
            _ => Segment::User,
        }
    }
}
