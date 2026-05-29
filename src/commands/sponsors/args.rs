use clap::Args;

use crate::types::SponsorStatus;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by status (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    pub status: Option<Vec<SponsorStatus>>,

    /// Output as JSON
    #[arg(long)]
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
