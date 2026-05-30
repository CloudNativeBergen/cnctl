use clap::Args;
use serde::Serialize;

use crate::types::{SponsorStatus, SponsorView};

#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    /// CRM View to use
    #[arg(long, value_enum, default_value_t = SponsorView::Pipeline)]
    pub view: SponsorView,

    /// Search across sponsor names, websites, and contact details
    #[arg(long = "search")]
    #[serde(rename = "searchQuery", skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by status (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<SponsorStatus>>,

    /// Filter by my assigned items
    #[arg(long = "mine")]
    #[serde(rename = "myAssignedOnly")]
    pub mine: bool,

    /// Filter by organizer (speaker ID)
    #[arg(long = "assigned-to")]
    #[serde(rename = "assignedTo", skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,

    /// Find leads without an assignee
    #[arg(long = "unassigned")]
    #[serde(rename = "unassignedOnly")]
    pub unassigned: bool,

    /// Filter by tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Filter by tier IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiers: Option<Vec<String>>,

    /// Output as JSON
    #[arg(long)]
    #[serde(skip)]
    pub json: bool,
}

#[derive(Args)]
pub struct EmailArgs {
    /// Sponsor-for-conference ID
    pub id: String,

    /// Template slug to use (interactive picker if omitted)
    #[arg(long)]
    pub template: Option<String>,

    /// Override the email subject
    #[arg(long)]
    pub subject: Option<String>,

    /// Use this message body directly (skip template selection)
    #[arg(long)]
    pub message: Option<String>,

    /// Open $EDITOR to edit the message before sending
    #[arg(long)]
    pub edit: bool,

    /// Preview the email without sending
    #[arg(long)]
    pub dry_run: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Sponsor name
    pub name: String,

    /// Website URL
    #[arg(long)]
    pub website: String,

    /// Primary contact name
    #[arg(long)]
    pub contact_name: Option<String>,

    /// Primary contact email
    #[arg(long)]
    pub contact_email: Option<String>,

    /// Status in the pipeline
    #[arg(long, default_value = "prospect", value_enum)]
    pub status: crate::types::SponsorStatus,

    /// Internal notes
    #[arg(long)]
    pub notes: Option<String>,
}

#[derive(Args)]
pub struct NoteArgs {
    /// Sponsor-for-conference ID
    pub id: String,

    /// Type of activity
    #[arg(long, default_value = "note", value_enum)]
    pub kind: crate::types::ActivityType,

    /// Description of the activity
    pub description: String,
}
