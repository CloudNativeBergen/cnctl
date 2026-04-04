use serde::{Deserialize, Serialize};

use super::null_to_vec;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub speakers: Vec<Speaker>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub topics: Vec<Topic>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub reviews: Vec<Review>,
    #[serde(default, rename = "_createdAt")]
    pub created_at: Option<String>,
    #[serde(default, rename = "_updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub outline: Option<String>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub description: Vec<serde_json::Value>,
    #[serde(default)]
    pub video: Option<String>,
}

/// Convert Portable Text blocks to plain text for terminal display.
pub fn portable_text_to_plain(blocks: &[serde_json::Value]) -> String {
    let mut paragraphs = Vec::new();

    for block in blocks {
        let block_type = block.get("_type").and_then(|t| t.as_str()).unwrap_or("");

        if block_type == "block" {
            let mut text = String::new();

            if let Some(children) = block.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    if let Some(span_text) = child.get("text").and_then(|t| t.as_str()) {
                        text.push_str(span_text);
                    }
                }
            }

            // Treat list items with a bullet prefix
            let style = block.get("listItem").and_then(|l| l.as_str());
            if let Some("bullet") = style {
                text = format!("  • {text}");
            } else if let Some("number") = style {
                text = format!("  - {text}");
            }

            if !text.is_empty() {
                paragraphs.push(text);
            }
        }
    }

    paragraphs.join("\n\n")
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Speaker {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    #[serde(default)]
    pub score: Option<ReviewScore>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub reviewer: Option<Reviewer>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewScore {
    #[serde(default)]
    pub content: f64,
    #[serde(default)]
    pub relevance: f64,
    #[serde(default)]
    pub speaker: f64,
}

impl ReviewScore {
    pub fn total(&self) -> f64 {
        self.content + self.relevance + self.speaker
    }
}

/// Input for `proposal.admin.submitReview` mutation.
#[derive(Debug, Serialize)]
pub struct ReviewInput {
    pub id: String,
    pub comment: String,
    pub score: ReviewScore,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reviewer {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full() {
        let json = serde_json::json!({
            "_id": "talk-1",
            "title": "Kubernetes Best Practices",
            "status": "submitted",
            "format": "presentation_25",
            "level": "intermediate",
            "language": "en",
            "outline": "We will cover...",
            "speakers": [
                {"_id": "sp-1", "name": "Alice", "email": "alice@example.com", "image": "https://img/a.jpg"}
            ],
            "topics": [
                {"title": "Kubernetes"},
                {"title": "DevOps"}
            ],
            "reviews": [
                {"score": {"content": 8.0, "relevance": 7.0, "speaker": 9.0}, "comment": "Great talk", "reviewer": {"name": "Bob"}}
            ]
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-1");
        assert_eq!(p.title, "Kubernetes Best Practices");
        assert_eq!(p.status, "submitted");
        assert_eq!(p.format.as_deref(), Some("presentation_25"));
        assert_eq!(p.speakers.len(), 1);
        assert_eq!(p.speakers[0].name, "Alice");
        assert_eq!(p.topics.len(), 2);
        assert_eq!(p.reviews.len(), 1);
        let score = p.reviews[0].score.as_ref().unwrap();
        assert!((score.total() - 24.0).abs() < f64::EPSILON);
        assert!((score.content - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deserialize_minimal() {
        let json = serde_json::json!({
            "_id": "talk-2",
            "title": "My Talk",
            "status": "draft"
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-2");
        assert!(p.format.is_none());
        assert!(p.speakers.is_empty());
        assert!(p.topics.is_empty());
        assert!(p.reviews.is_empty());
        assert!(p.outline.is_none());
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = serde_json::json!({
            "_id": "talk-3",
            "title": "Extra Fields Talk",
            "status": "accepted",
            "_createdAt": "2025-01-01T00:00:00Z",
            "_updatedAt": "2025-02-01T00:00:00Z",
            "description": [{"_type": "block", "children": []}],
            "scheduleInfo": {"room": "A", "slot": 1},
            "someNewField": "should be ignored"
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-3");
        assert_eq!(p.status, "accepted");
    }

    #[test]
    fn review_without_reviewer() {
        let json = serde_json::json!({
            "score": {"content": 5.0, "relevance": 4.0, "speaker": 3.0},
            "comment": "Needs more detail"
        });

        let r: Review = serde_json::from_value(json).unwrap();
        let score = r.score.as_ref().unwrap();
        assert!((score.total() - 12.0).abs() < f64::EPSILON);
        assert!(r.reviewer.is_none());
    }

    #[test]
    fn review_without_score() {
        let json = serde_json::json!({
            "comment": "No score given",
            "reviewer": {"name": "Carol"}
        });

        let r: Review = serde_json::from_value(json).unwrap();
        assert!(r.score.is_none());
        assert_eq!(r.reviewer.as_ref().unwrap().name, "Carol");
    }

    #[test]
    fn portable_text_paragraphs() {
        let blocks = vec![
            serde_json::json!({
                "_type": "block",
                "children": [{"_type": "span", "text": "First paragraph."}]
            }),
            serde_json::json!({
                "_type": "block",
                "children": [
                    {"_type": "span", "text": "Second "},
                    {"_type": "span", "text": "paragraph."}
                ]
            }),
        ];
        let result = portable_text_to_plain(&blocks);
        assert_eq!(result, "First paragraph.\n\nSecond paragraph.");
    }

    #[test]
    fn portable_text_with_list_items() {
        let blocks = vec![
            serde_json::json!({
                "_type": "block",
                "children": [{"_type": "span", "text": "Intro"}]
            }),
            serde_json::json!({
                "_type": "block",
                "listItem": "bullet",
                "children": [{"_type": "span", "text": "Item one"}]
            }),
            serde_json::json!({
                "_type": "block",
                "listItem": "bullet",
                "children": [{"_type": "span", "text": "Item two"}]
            }),
        ];
        let result = portable_text_to_plain(&blocks);
        assert_eq!(result, "Intro\n\n  • Item one\n\n  • Item two");
    }

    #[test]
    fn portable_text_empty() {
        assert_eq!(portable_text_to_plain(&[]), "");
    }

    #[test]
    fn null_vec_fields_deserialize() {
        let json = serde_json::json!({
            "_id": "talk-null",
            "title": "Null Vecs",
            "status": "draft",
            "speakers": null,
            "topics": null,
            "reviews": null,
            "description": null
        });

        let p: Proposal = serde_json::from_value(json).unwrap();
        assert_eq!(p.id, "talk-null");
        assert!(p.speakers.is_empty());
        assert!(p.topics.is_empty());
        assert!(p.reviews.is_empty());
        assert!(p.description.is_empty());
    }
}
