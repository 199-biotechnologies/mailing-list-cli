//! Resend webhook event shapes. See https://resend.com/docs/dashboard/webhooks
//! for the authoritative list.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResendEventType {
    #[serde(rename = "email.sent")]
    Sent,
    #[serde(rename = "email.delivered")]
    Delivered,
    #[serde(rename = "email.delivery_delayed")]
    DeliveryDelayed,
    #[serde(rename = "email.bounced")]
    Bounced,
    #[serde(rename = "email.complained")]
    Complained,
    #[serde(rename = "email.opened")]
    Opened,
    #[serde(rename = "email.clicked")]
    Clicked,
    #[serde(rename = "email.suppressed")]
    Suppressed,
    #[serde(rename = "email.failed")]
    Failed,
    #[serde(rename = "email.scheduled")]
    Scheduled,
    #[serde(other)]
    Unknown,
}

impl ResendEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sent => "email.sent",
            Self::Delivered => "email.delivered",
            Self::DeliveryDelayed => "email.delivery_delayed",
            Self::Bounced => "email.bounced",
            Self::Complained => "email.complained",
            Self::Opened => "email.opened",
            Self::Clicked => "email.clicked",
            Self::Suppressed => "email.suppressed",
            Self::Failed => "email.failed",
            Self::Scheduled => "email.scheduled",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendEvent {
    #[serde(rename = "type")]
    pub event_type: ResendEventType,
    pub created_at: String,
    pub data: ResendEventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendEventData {
    pub email_id: String,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub bounce: Option<BounceInfo>,
    #[serde(default)]
    pub click: Option<ClickInfo>,
    #[serde(default, rename = "complaint_type")]
    pub complaint_type: Option<String>,
    #[serde(default)]
    pub tags: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceInfo {
    #[serde(rename = "type")]
    pub bounce_type: String, // "Permanent" | "Transient"
    pub message: Option<String>,
    pub subtype: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickInfo {
    pub link: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_delivered_event() {
        let json = r#"{
            "type": "email.delivered",
            "created_at": "2026-04-08T12:00:00Z",
            "data": {
                "email_id": "em_test_1",
                "to": ["alice@example.com"],
                "subject": "Hi"
            }
        }"#;
        let ev: ResendEvent = serde_json::from_str(json).unwrap();
        assert_eq!(ev.event_type, ResendEventType::Delivered);
        assert_eq!(ev.data.email_id, "em_test_1");
    }

    #[test]
    fn deserializes_bounced_event_with_permanent_type() {
        let json = r#"{
            "type": "email.bounced",
            "created_at": "2026-04-08T12:00:00Z",
            "data": {
                "email_id": "em_test_2",
                "to": ["alice@example.com"],
                "bounce": {"type": "Permanent", "message": "mailbox full"}
            }
        }"#;
        let ev: ResendEvent = serde_json::from_str(json).unwrap();
        assert_eq!(ev.event_type, ResendEventType::Bounced);
        assert_eq!(ev.data.bounce.unwrap().bounce_type, "Permanent");
    }

    #[test]
    fn deserializes_clicked_event_with_link() {
        let json = r#"{
            "type": "email.clicked",
            "created_at": "2026-04-08T12:00:00Z",
            "data": {
                "email_id": "em_test_3",
                "to": ["alice@example.com"],
                "click": {"link": "https://example.com/cta", "ip_address": "1.2.3.4"}
            }
        }"#;
        let ev: ResendEvent = serde_json::from_str(json).unwrap();
        assert_eq!(ev.event_type, ResendEventType::Clicked);
        assert_eq!(ev.data.click.unwrap().link, "https://example.com/cta");
    }
}
