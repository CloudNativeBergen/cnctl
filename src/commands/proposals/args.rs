use std::fmt;

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortField {
    Created,
    Title,
    Speaker,
    Rating,
    Reviews,
    Status,
}

impl fmt::Display for SortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Title => write!(f, "title"),
            Self::Speaker => write!(f, "speaker"),
            Self::Rating => write!(f, "rating"),
            Self::Reviews => write!(f, "reviews"),
            Self::Status => write!(f, "status"),
        }
    }
}

#[derive(Args)]
pub struct ListArgs {
    /// Output as JSON (non-interactive)
    #[arg(long)]
    pub json: bool,

    /// Filter by status (comma-separated, e.g. submitted,accepted)
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by format (comma-separated, e.g. `presentation_40,lightning_10`)
    #[arg(long)]
    pub format: Option<String>,

    /// Sort by field
    #[arg(long, value_enum, default_value_t = SortField::Created)]
    pub sort: SortField,

    /// Sort ascending instead of descending
    #[arg(long)]
    pub asc: bool,
}

#[derive(Args)]
pub struct ReviewArgs {
    /// Proposal ID
    pub id: String,

    /// Content score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5))]
    pub content: Option<u8>,

    /// Relevance score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5))]
    pub relevance: Option<u8>,

    /// Speaker score (1–5)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=5))]
    pub speaker: Option<u8>,

    /// Review comment
    #[arg(long)]
    pub comment: Option<String>,
}

impl ListArgs {
    pub fn has_cli_filters(&self) -> bool {
        self.status.is_some() || self.format.is_some() || self.sort != SortField::Created || self.asc
    }
}
