use clap::{Args, Subcommand};
use serde::Serialize;

use crate::types::{ProposalSortBy, ProposalStatus, SortOrder, SpeakerFlag};

#[derive(Args)]
pub struct SpeakerArgs {
    #[command(subcommand)]
    pub command: SpeakerCommand,
}

#[derive(Subcommand)]
pub enum SpeakerCommand {
    /// List speakers (defaults to accepted/confirmed speakers)
    List(ListArgs),
    /// Show full speaker details
    Get {
        /// Speaker ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a new speaker profile
    Add(CreateArgs),
    /// Delete a speaker
    Delete {
        /// Speaker ID
        id: String,
    },
    /// Send a broadcast email to ALL speakers (use with caution!)
    Broadcast {
        /// Email subject
        #[arg(long)]
        subject: String,

        /// Email message (plain text)
        #[arg(long)]
        message: String,
    },
    /// Sync with newsletter audience
    SyncAudience,
}

#[derive(Args, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArgs {
    /// Output as JSON
    #[arg(long)]
    #[serde(skip)]
    pub json: bool,

    /// Search across names and emails
    #[arg(long = "search")]
    #[serde(rename = "query", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Filter by proposal status (comma-separated, defaults to accepted,confirmed)
    #[arg(long, value_delimiter = ',', value_enum)]
    #[serde(skip)]
    pub status: Option<Vec<ProposalStatus>>,

    /// Sort by field
    #[arg(long = "sort", value_enum, default_value_t = ProposalSortBy::Speaker)]
    #[serde(skip)]
    pub sort: ProposalSortBy,

    /// Sort order
    #[arg(long = "order", value_enum, default_value_t = SortOrder::Asc)]
    #[serde(skip)]
    pub order: SortOrder,
}

#[derive(Args, Serialize)]
pub struct CreateArgs {
    /// Speaker name
    pub name: String,

    /// Speaker email
    pub email: String,

    /// Job title
    #[arg(long)]
    pub title: Option<String>,

    /// Company name
    #[arg(long)]
    pub company: Option<String>,

    /// Biography (plain text)
    #[arg(long)]
    pub bio: Option<String>,

    /// Image URL
    #[arg(long)]
    pub image: Option<String>,

    /// Social links (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub links: Option<Vec<String>>,

    /// Speaker flags (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    pub flags: Option<Vec<SpeakerFlag>>,
}
