use crate::types::{Proposal, ProposalFormat, ProposalStatus, ReviewScore};

use super::args::{ListArgs, SortField};

pub const STATUSES: &[ProposalStatus] = &[
    ProposalStatus::Submitted,
    ProposalStatus::Accepted,
    ProposalStatus::Confirmed,
    ProposalStatus::Waitlisted,
    ProposalStatus::Rejected,
    ProposalStatus::Withdrawn,
    ProposalStatus::Draft,
];

pub const FORMATS: &[ProposalFormat] = &[
    ProposalFormat::Lightning10,
    ProposalFormat::Presentation20,
    ProposalFormat::Presentation25,
    ProposalFormat::Presentation40,
    ProposalFormat::Presentation45,
    ProposalFormat::Workshop120,
    ProposalFormat::Workshop240,
];

pub const SORT_FIELDS: &[SortField] = &[
    SortField::Created,
    SortField::Title,
    SortField::Speaker,
    SortField::Rating,
    SortField::Reviews,
    SortField::Status,
];

pub const SORT_LABELS: &[&str] = &[
    "Date created",
    "Title",
    "Speaker name",
    "Average rating",
    "Review count",
    "Status",
];

#[derive(Clone)]
pub struct Filters {
    pub statuses: Vec<ProposalStatus>,
    pub formats: Vec<ProposalFormat>,
    pub sort_by: SortField,
    pub sort_asc: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            statuses: vec![
                ProposalStatus::Submitted,
                ProposalStatus::Accepted,
                ProposalStatus::Confirmed,
            ],
            formats: vec![],
            sort_by: SortField::Created,
            sort_asc: false,
        }
    }
}

impl From<&ListArgs> for Filters {
    fn from(args: &ListArgs) -> Self {
        let statuses = args.status.as_deref().map_or_else(Vec::new, |s| {
            s.split(',')
                .filter_map(|v| {
                    let v = v.trim();
                    STATUSES.iter().find(|st| st.to_string().eq_ignore_ascii_case(v)).copied()
                })
                .collect()
        });
        let formats = args.format.as_deref().map_or_else(Vec::new, |s| {
            s.split(',')
                .filter_map(|v| {
                    let v = v.trim();
                    FORMATS.iter().find(|f| f.api_name().eq_ignore_ascii_case(v)).copied()
                })
                .collect()
        });
        Self {
            statuses,
            formats,
            sort_by: args.sort,
            sort_asc: args.asc,
        }
    }
}

pub fn apply_filters<'a>(proposals: &'a [Proposal], filters: &Filters) -> Vec<&'a Proposal> {
    let mut filtered: Vec<&Proposal> = proposals
        .iter()
        .filter(|p| {
            if !filters.statuses.is_empty() && !filters.statuses.contains(&p.status) {
                return false;
            }
            if !filters.formats.is_empty() {
                match p.format {
                    Some(fmt) if filters.formats.contains(&fmt) => {}
                    Some(_) | None => return false,
                }
            }
            true
        })
        .collect();

    match filters.sort_by {
        SortField::Title => {
            filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        SortField::Speaker => {
            filtered.sort_by(|a, b| {
                let sa = a.speakers.first().map_or("", |s| s.name.as_str());
                let sb = b.speakers.first().map_or("", |s| s.name.as_str());
                sa.to_lowercase().cmp(&sb.to_lowercase())
            });
        }
        SortField::Rating => {
            filtered.sort_by(|a, b| {
                avg_rating(a)
                    .partial_cmp(&avg_rating(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortField::Reviews => filtered.sort_by(|a, b| a.reviews.len().cmp(&b.reviews.len())),
        SortField::Status => filtered.sort_by(|a, b| a.status.cmp(&b.status)),
        SortField::Created => {
            filtered.sort_by(|a, b| {
                let ca = a.created_at.as_deref().unwrap_or("");
                let cb = b.created_at.as_deref().unwrap_or("");
                ca.cmp(cb)
            });
        }
    }

    if !filters.sort_asc {
        filtered.reverse();
    }

    filtered
}

pub fn avg_rating(p: &Proposal) -> f64 {
    if p.reviews.is_empty() {
        return 0.0;
    }
    let total: f64 = p
        .reviews
        .iter()
        .filter_map(|r| r.score.as_ref())
        .map(ReviewScore::total)
        .sum();
    let count = p
        .reviews
        .iter()
        .filter(|r| r.score.is_some())
        .count()
        .max(1);
    #[allow(clippy::cast_precision_loss)]
    {
        total / count as f64
    }
}
