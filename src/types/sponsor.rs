use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use super::null_to_vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum SponsorStatus {
    Prospect,
    Contacted,
    Negotiating,
    ClosedWon,
    ClosedLost,
    /// Catch-all for unknown values from the API.
    #[value(skip)]
    #[serde(other)]
    Unknown,
}

impl<'de> Deserialize<'de> for SponsorStatus {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        enum Helper {
            Prospect,
            Contacted,
            Negotiating,
            ClosedWon,
            ClosedLost,
            #[serde(other)]
            Unknown,
        }
        Ok(match Helper::deserialize(deserializer)? {
            Helper::Prospect => Self::Prospect,
            Helper::Contacted => Self::Contacted,
            Helper::Negotiating => Self::Negotiating,
            Helper::ClosedWon => Self::ClosedWon,
            Helper::ClosedLost => Self::ClosedLost,
            Helper::Unknown => Self::Unknown,
        })
    }
}

impl fmt::Display for SponsorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prospect => write!(f, "prospect"),
            Self::Contacted => write!(f, "contacted"),
            Self::Negotiating => write!(f, "negotiating"),
            Self::ClosedWon => write!(f, "closed-won"),
            Self::ClosedLost => write!(f, "closed-lost"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorForConference {
    #[serde(rename = "_id")]
    pub id: String,
    pub status: SponsorStatus,
    #[serde(default)]
    pub contract_status: Option<String>,
    #[serde(default)]
    pub invoice_status: Option<String>,
    #[serde(default)]
    pub sponsor: Option<SponsorRef>,
    #[serde(default)]
    pub tier: Option<TierRef>,
    #[serde(default)]
    pub assigned_to: Option<AssignedTo>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub contact_persons: Vec<ContactPerson>,
    #[serde(default)]
    pub billing: Option<Billing>,
    #[serde(default)]
    pub contract_value: Option<f64>,
    #[serde(default)]
    pub contract_currency: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, deserialize_with = "null_to_vec")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub contract_signed_at: Option<String>,
    #[serde(default)]
    pub invoice_sent_at: Option<String>,
    #[serde(default)]
    pub invoice_paid_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorRef {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierRef {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignedTo {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactPerson {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Billing {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full() {
        let json = serde_json::json!({
            "_id": "sfc-1",
            "status": "closed-won",
            "contractStatus": "contract-signed",
            "invoiceStatus": "paid",
            "sponsor": {"_id": "sp-1", "name": "Acme Corp", "website": "https://acme.com"},
            "tier": {"_id": "tier-1", "title": "Gold"},
            "assignedTo": {"_id": "org-1", "name": "Hans"},
            "contactPersons": [
                {"name": "Jane", "email": "jane@acme.com", "phone": "+47123", "role": "CTO", "isPrimary": true}
            ],
            "billing": {"email": "billing@acme.com", "reference": "PO-123"},
            "contractValue": 50000.0,
            "contractCurrency": "NOK",
            "notes": "Important sponsor",
            "tags": ["returning", "vip"],
            "contractSignedAt": "2025-06-01",
            "invoiceSentAt": "2025-06-15",
            "invoicePaidAt": "2025-07-01"
        });

        let s: SponsorForConference = serde_json::from_value(json).unwrap();
        assert_eq!(s.id, "sfc-1");
        assert_eq!(s.status, SponsorStatus::ClosedWon);
        assert_eq!(s.contract_status.as_deref(), Some("contract-signed"));
        assert_eq!(s.sponsor.as_ref().unwrap().name, "Acme Corp");
        assert_eq!(s.tier.as_ref().unwrap().title, "Gold");
        assert_eq!(s.assigned_to.as_ref().unwrap().name, "Hans");
        assert_eq!(s.contact_persons.len(), 1);
        assert_eq!(s.contact_persons[0].is_primary, Some(true));
        assert_eq!(
            s.billing.as_ref().unwrap().reference.as_deref(),
            Some("PO-123")
        );
        assert_eq!(s.contract_value, Some(50000.0));
        assert_eq!(s.tags, vec!["returning", "vip"]);
    }

    #[test]
    fn deserialize_minimal() {
        let json = serde_json::json!({
            "_id": "sfc-2",
            "status": "prospect"
        });

        let s: SponsorForConference = serde_json::from_value(json).unwrap();
        assert_eq!(s.id, "sfc-2");
        assert_eq!(s.status, SponsorStatus::Prospect);
        assert!(s.sponsor.is_none());
        assert!(s.tier.is_none());
        assert!(s.contact_persons.is_empty());
        assert!(s.tags.is_empty());
        assert!(s.contract_value.is_none());
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = serde_json::json!({
            "_id": "sfc-3",
            "status": "contacted",
            "registrationToken": "abc123",
            "registrationComplete": true,
            "addons": [{"_id": "addon-1", "title": "Extra booth"}],
            "signatureStatus": "pending",
            "futureField": 42
        });

        let s: SponsorForConference = serde_json::from_value(json).unwrap();
        assert_eq!(s.id, "sfc-3");
        assert_eq!(s.status, SponsorStatus::Contacted);
    }

    #[test]
    fn contact_person_minimal() {
        let json = serde_json::json!({"name": "Bob"});

        let c: ContactPerson = serde_json::from_value(json).unwrap();
        assert_eq!(c.name, "Bob");
        assert!(c.email.is_none());
        assert!(c.is_primary.is_none());
    }

    #[test]
    fn null_vec_fields_deserialize() {
        let json = serde_json::json!({
            "_id": "sfc-null",
            "status": "prospect",
            "contactPersons": null,
            "tags": null
        });

        let s: SponsorForConference = serde_json::from_value(json).unwrap();
        assert_eq!(s.id, "sfc-null");
        assert!(s.contact_persons.is_empty());
        assert!(s.tags.is_empty());
    }

    #[test]
    fn unknown_status_deserializes_gracefully() {
        let json = serde_json::json!({
            "_id": "sfc-future",
            "status": "archived",
        });

        let s: SponsorForConference = serde_json::from_value(json).unwrap();
        assert_eq!(s.status, SponsorStatus::Unknown);
        assert_eq!(s.status.to_string(), "unknown");
    }
}
