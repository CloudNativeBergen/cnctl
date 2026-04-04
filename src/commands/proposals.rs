use anyhow::Result;
use clap::Args;
use colored::Colorize;
use console::{Key, Term};
use dialoguer::{Confirm, FuzzySelect, Input, MultiSelect, Select};

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::{Proposal, ReviewInput, ReviewScore};
use crate::{config, ui};

// ---------------------------------------------------------------------------
// CLI argument types (clap::Args — used directly from main.rs)
// ---------------------------------------------------------------------------

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

    /// Sort by field: created, title, speaker, rating, reviews, status
    #[arg(long, default_value = "created")]
    pub sort: String,

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
    fn has_cli_filters(&self) -> bool {
        self.status.is_some() || self.format.is_some() || self.sort != "created" || self.asc
    }
}

// ---------------------------------------------------------------------------
// Domain: filter & sort logic (pure, no I/O)
// ---------------------------------------------------------------------------

const STATUSES: &[&str] = &[
    "submitted",
    "accepted",
    "confirmed",
    "waitlisted",
    "rejected",
    "withdrawn",
    "draft",
];

const FORMATS: &[&str] = &[
    "lightning_10",
    "presentation_20",
    "presentation_25",
    "presentation_40",
    "presentation_45",
    "workshop_120",
    "workshop_240",
];

const SORT_FIELDS: &[&str] = &["created", "title", "speaker", "rating", "reviews", "status"];

#[derive(Clone)]
struct Filters {
    statuses: Vec<String>,
    formats: Vec<String>,
    sort_by: String,
    sort_asc: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            statuses: vec!["submitted".into(), "accepted".into(), "confirmed".into()],
            formats: vec![],
            sort_by: "created".into(),
            sort_asc: false,
        }
    }
}

impl From<&ListArgs> for Filters {
    fn from(args: &ListArgs) -> Self {
        let parse_csv = |s: &str| s.split(',').map(|v| v.trim().to_string()).collect();
        Self {
            statuses: args.status.as_deref().map_or_else(Vec::new, parse_csv),
            formats: args.format.as_deref().map_or_else(Vec::new, parse_csv),
            sort_by: args.sort.clone(),
            sort_asc: args.asc,
        }
    }
}

fn apply_filters<'a>(proposals: &'a [Proposal], filters: &Filters) -> Vec<&'a Proposal> {
    let mut filtered: Vec<&Proposal> = proposals
        .iter()
        .filter(|p| {
            if !filters.statuses.is_empty() && !filters.statuses.contains(&p.status) {
                return false;
            }
            if !filters.formats.is_empty() {
                let fmt = p.format.as_deref().unwrap_or("");
                if !filters.formats.iter().any(|f| f == fmt) {
                    return false;
                }
            }
            true
        })
        .collect();

    match filters.sort_by.as_str() {
        "title" => filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
        "speaker" => filtered.sort_by(|a, b| {
            let sa = a.speakers.first().map_or("", |s| s.name.as_str());
            let sb = b.speakers.first().map_or("", |s| s.name.as_str());
            sa.to_lowercase().cmp(&sb.to_lowercase())
        }),
        "rating" => filtered.sort_by(|a, b| {
            avg_rating(a)
                .partial_cmp(&avg_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "reviews" => filtered.sort_by(|a, b| a.reviews.len().cmp(&b.reviews.len())),
        "status" => filtered.sort_by(|a, b| a.status.cmp(&b.status)),
        _ => filtered.sort_by(|a, b| {
            let ca = a.created_at.as_deref().unwrap_or("");
            let cb = b.created_at.as_deref().unwrap_or("");
            ca.cmp(cb)
        }),
    }

    if !filters.sort_asc {
        filtered.reverse();
    }

    filtered
}

fn avg_rating(p: &Proposal) -> f64 {
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

// ---------------------------------------------------------------------------
// Display helpers (formatting for terminal output)
// ---------------------------------------------------------------------------

const TABLE_HEADER: &str = "STATUS       FORMAT           TITLE · SPEAKER";

fn format_item(p: &Proposal) -> String {
    let speakers: Vec<&str> = p.speakers.iter().map(|s| s.name.as_str()).collect();
    let speaker_str = speakers.join(", ");
    let format = display::humanize_format(p.format.as_deref().unwrap_or("-"));
    let status = display::pad_and_colorize_status(&p.status, 12);

    let prefix_len = 12 + 1 + 16 + 1;
    let prefix = format!("{status} {format:<16} ");
    let remaining = ui::term_width().saturating_sub(prefix_len + 4);
    let title_budget = remaining * 2 / 3;
    let speaker_budget = remaining.saturating_sub(title_budget + 3);

    let title = ui::truncate(&p.title, title_budget);
    if speaker_str.is_empty() {
        format!("{prefix}{title}")
    } else {
        let speaker = ui::truncate(&speaker_str, speaker_budget);
        format!("{prefix}{title} · {speaker}")
    }
}

fn filter_summary(filters: &Filters) -> String {
    let status_part = if filters.statuses.is_empty() {
        "all statuses".into()
    } else {
        filters.statuses.join(", ")
    };
    let format_part = if filters.formats.is_empty() {
        "all formats".into()
    } else {
        filters
            .formats
            .iter()
            .map(|f| display::humanize_format(f))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let dir = if filters.sort_asc { "↑" } else { "↓" };
    format!(
        "status: {status_part} | format: {format_part} | sort: {}{dir}",
        filters.sort_by
    )
}

// ---------------------------------------------------------------------------
// Interactive filter menu
// ---------------------------------------------------------------------------

fn show_filter_menu(filters: &mut Filters) -> Result<()> {
    let term = Term::stderr();
    term.clear_screen()?;

    // Status filter
    let status_defaults: Vec<bool> = STATUSES
        .iter()
        .map(|s| filters.statuses.contains(&s.to_string()))
        .collect();
    let status_labels: Vec<String> = STATUSES
        .iter()
        .map(|s| display::humanize_status(s).to_string())
        .collect();

    println!(
        "{}",
        "Filter by status (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&status_labels)
        .defaults(&status_defaults)
        .interact()?;
    filters.statuses = selected.iter().map(|&i| STATUSES[i].to_string()).collect();

    // Format filter
    let format_defaults: Vec<bool> = FORMATS
        .iter()
        .map(|f| {
            if filters.formats.is_empty() {
                true
            } else {
                filters.formats.contains(&f.to_string())
            }
        })
        .collect();
    let format_labels: Vec<String> = FORMATS
        .iter()
        .map(|f| display::humanize_format(f).to_string())
        .collect();

    println!(
        "\n{}",
        "Filter by format (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&format_labels)
        .defaults(&format_defaults)
        .interact()?;
    filters.formats = if selected.len() == FORMATS.len() {
        vec![]
    } else {
        selected.iter().map(|&i| FORMATS[i].to_string()).collect()
    };

    // Sort field
    let sort_labels: Vec<&str> = SORT_FIELDS
        .iter()
        .map(|&s| match s {
            "created" => "Date created",
            "title" => "Title",
            "speaker" => "Speaker name",
            "rating" => "Average rating",
            "reviews" => "Review count",
            "status" => "Status",
            _ => s,
        })
        .collect();
    let sort_default = SORT_FIELDS
        .iter()
        .position(|&s| s == filters.sort_by)
        .unwrap_or(0);

    println!("\n{}", "Sort by:".bold());
    let sort_idx = Select::new()
        .items(&sort_labels)
        .default(sort_default)
        .interact()?;
    filters.sort_by = SORT_FIELDS[sort_idx].to_string();

    // Sort direction
    let dir_default = usize::from(filters.sort_asc);
    println!("\n{}", "Sort direction:".bold());
    let dir_idx = Select::new()
        .items(["Descending ↓", "Ascending ↑"])
        .default(dir_default)
        .interact()?;
    filters.sort_asc = dir_idx == 1;

    term.clear_screen()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// API helpers
// ---------------------------------------------------------------------------

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<Proposal>> {
    client.query("proposal.admin.list", None).await
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<Proposal> {
    let input = serde_json::json!({ "id": id });
    client.query("proposal.admin.getById", Some(&input)).await
}

pub async fn submit_review(client: &TrpcClient, input: &ReviewInput) -> Result<serde_json::Value> {
    client
        .mutate("proposal.admin.submitReview", &serde_json::to_value(input)?)
        .await
}

// ---------------------------------------------------------------------------
// Command entry points
// ---------------------------------------------------------------------------

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching proposals…");
    let all = fetch_all(&client).await?;
    sp.finish_and_clear();

    if args.json {
        list_json(&all, &args)
    } else if args.has_cli_filters() {
        list_plain(&all, &args);
        Ok(())
    } else {
        list_interactive(&client, &all).await
    }
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching proposal…");
    let proposal = fetch_one(&client, id).await?;
    sp.finish_and_clear();

    if json {
        println!("{}", serde_json::to_string_pretty(&proposal)?);
    } else {
        display::print_proposal_detail(&proposal);
    }
    Ok(())
}

pub async fn review(args: ReviewArgs) -> Result<()> {
    let client = require_client()?;
    let reviewer_name = config::load().ok().and_then(|c| c.name);

    let sp = ui::spinner("Fetching proposal…");
    let proposal = fetch_one(&client, &args.id).await?;
    sp.finish_and_clear();

    display::print_proposal_detail(&proposal);
    println!();

    // If all scores and comment are provided, submit non-interactively
    if let (Some(content), Some(relevance), Some(speaker), Some(comment)) =
        (args.content, args.relevance, args.speaker, args.comment)
    {
        let input = ReviewInput {
            id: args.id,
            comment,
            score: ReviewScore {
                content: f64::from(content),
                relevance: f64::from(relevance),
                speaker: f64::from(speaker),
            },
        };

        let sp = ui::spinner("Submitting review…");
        submit_review(&client, &input).await?;
        sp.finish_and_clear();

        println!("Review submitted ({:.0}/15)", input.score.total());
    } else {
        prompt_and_submit_review(&client, &proposal, reviewer_name.as_deref()).await?;
    }

    Ok(())
}

async fn prompt_and_submit_review(
    client: &TrpcClient,
    proposal: &Proposal,
    reviewer_name: Option<&str>,
) -> Result<()> {
    // Find the user's existing review to pre-fill defaults
    let existing = reviewer_name.and_then(|name| {
        proposal.reviews.iter().find(|r| {
            r.reviewer
                .as_ref()
                .is_some_and(|rev| rev.name.eq_ignore_ascii_case(name))
        })
    });

    if existing.is_some() {
        println!("{}", "Updating your existing review.".dimmed());
    }

    let prev_score = existing.and_then(|r| r.score.as_ref());
    let prev_comment = existing.and_then(|r| r.comment.as_deref()).unwrap_or("");

    // Prompt scores (Esc to cancel at any step)
    let Some(content) = prompt_score("Content", score_default(prev_score, |s| s.content))? else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };
    let Some(relevance) = prompt_score("Relevance", score_default(prev_score, |s| s.relevance))?
    else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };
    let Some(speaker) = prompt_score("Speaker", score_default(prev_score, |s| s.speaker))? else {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    };

    let comment: String = Input::new()
        .with_prompt("Comment")
        .with_initial_text(prev_comment)
        .allow_empty(true)
        .interact_text()?;

    // Show summary and confirm
    let total = f64::from(content) + f64::from(relevance) + f64::from(speaker);
    println!(
        "\n  Content: {content}  Relevance: {relevance}  Speaker: {speaker}  Total: {total:.0}/15"
    );
    if !comment.is_empty() {
        println!("  Comment: {comment}");
    }

    if !Confirm::new()
        .with_prompt("Submit review?")
        .default(true)
        .interact()?
    {
        println!("{}", "Review cancelled.".dimmed());
        return Ok(());
    }

    let input = ReviewInput {
        id: proposal.id.clone(),
        comment,
        score: ReviewScore {
            content: f64::from(content),
            relevance: f64::from(relevance),
            speaker: f64::from(speaker),
        },
    };

    let sp = ui::spinner("Submitting review…");
    submit_review(client, &input).await?;
    sp.finish_and_clear();

    println!(
        "{} Review submitted ({:.0}/15)",
        "✔".green().bold(),
        input.score.total()
    );
    Ok(())
}

fn score_default(prev: Option<&ReviewScore>, f: impl Fn(&ReviewScore) -> f64) -> usize {
    prev.map_or(2, |s| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v = f(s) as usize;
        v.saturating_sub(1).min(4)
    })
}

const SCORE_LABELS: &[&str] = &[
    "1 - Poor",
    "2 - Fair",
    "3 - Good",
    "4 - Very good",
    "5 - Excellent",
];

fn prompt_score(category: &str, default: usize) -> Result<Option<u8>> {
    let selection = Select::new()
        .with_prompt(format!("{category} (1–5, esc to cancel)"))
        .items(SCORE_LABELS)
        .default(default)
        .interact_opt()?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(selection.map(|idx| (idx + 1) as u8))
}

// ---------------------------------------------------------------------------
// Output modes
// ---------------------------------------------------------------------------

fn list_json(all: &[Proposal], args: &ListArgs) -> Result<()> {
    let filters = Filters::from(args);
    let filtered = apply_filters(all, &filters);
    println!("{}", serde_json::to_string_pretty(&filtered)?);
    Ok(())
}

fn list_plain(all: &[Proposal], args: &ListArgs) {
    let filters = Filters::from(args);
    let filtered = apply_filters(all, &filters);

    if filtered.is_empty() {
        println!("No proposals match the given filters.");
        return;
    }

    println!("{TABLE_HEADER}");
    for p in &filtered {
        println!("{}", format_item(p));
    }
}

async fn list_interactive(client: &TrpcClient, all_proposals: &[Proposal]) -> Result<()> {
    if all_proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut filters = Filters::default();
    let mut cursor = 0usize;

    loop {
        let filtered = apply_filters(all_proposals, &filters);
        let summary = filter_summary(&filters);

        if filtered.is_empty() {
            println!("No proposals match current filters. Press enter to adjust filters.");
            show_filter_menu(&mut filters)?;
            continue;
        }

        let menu_label = format!("⚙ Filter & Sort  ({summary})");
        let mut items: Vec<String> = vec![menu_label];
        items.extend(filtered.iter().map(|p| format_item(p)));

        let default = (cursor + 1).min(items.len() - 1);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{}/{} proposals\n  {TABLE_HEADER}\n  {hints}",
                filtered.len(),
                all_proposals.len()
            ))
            .items(&items)
            .default(default)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(0) => {
                show_filter_menu(&mut filters)?;
                cursor = 0;
            }
            Some(idx) => {
                cursor = idx - 1;
                let proposal_ids: Vec<&str> = filtered.iter().map(|p| p.id.as_str()).collect();
                cursor = show_detail_loop(client, &proposal_ids, cursor).await?;
            }
            None => break,
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Detail view with prev/next navigation
// ---------------------------------------------------------------------------

async fn show_detail_loop(
    client: &TrpcClient,
    proposal_ids: &[&str],
    start: usize,
) -> Result<usize> {
    let reviewer_name = config::load().ok().and_then(|c| c.name);
    let mut idx = start;
    let total = proposal_ids.len();

    loop {
        let sp = ui::spinner("Loading…");
        let proposal = fetch_one(client, proposal_ids[idx]).await?;
        sp.finish_and_clear();

        let content = display::render_proposal_detail(&proposal);

        // Build nav hints — scroll hints added dynamically by the pager
        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        // Use the longest possible hint string for viewport sizing
        let mut nav_full = nav.clone();
        nav_full.extend(["↑↓/jk scroll", "^u/^d half-page", "r review", "q/esc back"]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        // Build the actual footer shown to the user
        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
        nav.push("r review");
        nav.push("q/esc back");
        let footer = nav.join(" · ").dimmed().to_string();

        loop {
            let header = if pager.is_scrollable() {
                format!(
                    "[{}/{}] ↕ {}/{}",
                    idx + 1,
                    total,
                    pager.scroll_offset() + 1,
                    pager.line_count()
                )
            } else {
                format!("[{}/{}]", idx + 1, total)
            };

            pager.render(&header.dimmed().to_string(), &footer)?;

            match pager.handle_key()? {
                ui::pager::Action::Redraw => {}
                ui::pager::Action::Custom(key) => match key {
                    Key::ArrowLeft | Key::Char('h') => {
                        idx = idx.saturating_sub(1);
                        break;
                    }
                    Key::ArrowRight | Key::Char('l') => {
                        if idx + 1 < total {
                            idx += 1;
                        }
                        break;
                    }
                    Key::Char('r') => {
                        println!();
                        prompt_and_submit_review(client, &proposal, reviewer_name.as_deref())
                            .await?;
                        break;
                    }
                    Key::Escape | Key::Char('q') => {
                        pager.clear()?;
                        return Ok(idx);
                    }
                    _ => {}
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proposal(id: &str, title: &str, status: &str, format: &str) -> Proposal {
        serde_json::from_value(serde_json::json!({
            "_id": id,
            "title": title,
            "status": status,
            "format": format,
            "speakers": [],
            "topics": [],
            "reviews": [],
        }))
        .unwrap()
    }

    fn make_proposal_with_speaker(id: &str, title: &str, status: &str, speaker: &str) -> Proposal {
        serde_json::from_value(serde_json::json!({
            "_id": id,
            "title": title,
            "status": status,
            "speakers": [{"_id": "s1", "name": speaker}],
            "topics": [],
            "reviews": [],
        }))
        .unwrap()
    }

    fn make_proposal_with_reviews(id: &str, title: &str, scores: &[(f64, f64, f64)]) -> Proposal {
        let reviews: Vec<serde_json::Value> = scores
            .iter()
            .map(|(c, r, s)| {
                serde_json::json!({
                    "score": {"content": c, "relevance": r, "speaker": s},
                    "reviewer": {"name": "Reviewer"}
                })
            })
            .collect();
        serde_json::from_value(serde_json::json!({
            "_id": id,
            "title": title,
            "status": "submitted",
            "speakers": [],
            "topics": [],
            "reviews": reviews,
        }))
        .unwrap()
    }

    fn test_proposals() -> Vec<Proposal> {
        vec![
            make_proposal("1", "Kubernetes Intro", "submitted", "presentation_40"),
            make_proposal("2", "Service Mesh", "accepted", "presentation_20"),
            make_proposal("3", "Lightning Demo", "submitted", "lightning_10"),
            make_proposal("4", "Workshop K8s", "rejected", "workshop_120"),
            make_proposal("5", "Observability", "confirmed", "presentation_40"),
        ]
    }

    #[test]
    fn filter_by_status_submitted() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec!["submitted".into()],
            ..Filters::default()
        };
        let result = apply_filters(&proposals, &filters);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|p| p.status == "submitted"));
    }

    #[test]
    fn filter_by_multiple_statuses() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec!["accepted".into(), "confirmed".into()],
            ..Filters::default()
        };
        let result = apply_filters(&proposals, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_format() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec![],
            formats: vec!["presentation_40".into()],
            ..Filters::default()
        };
        let result = apply_filters(&proposals, &filters);
        assert_eq!(result.len(), 2);
        assert!(
            result
                .iter()
                .all(|p| p.format.as_deref() == Some("presentation_40"))
        );
    }

    #[test]
    fn filter_empty_statuses_shows_all() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            ..Filters::default()
        };
        let result = apply_filters(&proposals, &filters);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec!["waitlisted".into()],
            ..Filters::default()
        };
        let result = apply_filters(&proposals, &filters);
        assert!(result.is_empty());
    }

    #[test]
    fn sort_by_title_asc() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            sort_by: "title".into(),
            sort_asc: true,
        };
        let result = apply_filters(&proposals, &filters);
        let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "Kubernetes Intro",
                "Lightning Demo",
                "Observability",
                "Service Mesh",
                "Workshop K8s"
            ]
        );
    }

    #[test]
    fn sort_by_title_desc() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            sort_by: "title".into(),
            sort_asc: false,
        };
        let result = apply_filters(&proposals, &filters);
        let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles[0], "Workshop K8s");
        assert_eq!(titles[4], "Kubernetes Intro");
    }

    #[test]
    fn sort_by_speaker() {
        let proposals = vec![
            make_proposal_with_speaker("1", "Talk A", "submitted", "Charlie"),
            make_proposal_with_speaker("2", "Talk B", "submitted", "Alice"),
            make_proposal_with_speaker("3", "Talk C", "submitted", "Bob"),
        ];
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            sort_by: "speaker".into(),
            sort_asc: true,
        };
        let result = apply_filters(&proposals, &filters);
        let speakers: Vec<&str> = result.iter().map(|p| p.speakers[0].name.as_str()).collect();
        assert_eq!(speakers, vec!["Alice", "Bob", "Charlie"]);
    }

    #[test]
    fn sort_by_rating_desc() {
        let proposals = vec![
            make_proposal_with_reviews("1", "Low rated", &[(1.0, 1.0, 1.0)]),
            make_proposal_with_reviews("2", "High rated", &[(5.0, 5.0, 5.0)]),
            make_proposal_with_reviews("3", "Medium rated", &[(3.0, 3.0, 3.0)]),
        ];
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            sort_by: "rating".into(),
            sort_asc: false,
        };
        let result = apply_filters(&proposals, &filters);
        let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, vec!["High rated", "Medium rated", "Low rated"]);
    }

    #[test]
    fn sort_by_reviews_count() {
        let proposals = vec![
            make_proposal_with_reviews("1", "No reviews", &[]),
            make_proposal_with_reviews("2", "Two reviews", &[(3.0, 3.0, 3.0), (4.0, 4.0, 4.0)]),
            make_proposal_with_reviews("3", "One review", &[(5.0, 5.0, 5.0)]),
        ];
        let filters = Filters {
            statuses: vec![],
            formats: vec![],
            sort_by: "reviews".into(),
            sort_asc: false,
        };
        let result = apply_filters(&proposals, &filters);
        let counts: Vec<usize> = result.iter().map(|p| p.reviews.len()).collect();
        assert_eq!(counts, vec![2, 1, 0]);
    }

    #[test]
    fn filter_and_sort_combined() {
        let proposals = test_proposals();
        let filters = Filters {
            statuses: vec!["submitted".into()],
            formats: vec![],
            sort_by: "title".into(),
            sort_asc: true,
        };
        let result = apply_filters(&proposals, &filters);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Kubernetes Intro");
        assert_eq!(result[1].title, "Lightning Demo");
    }

    #[test]
    fn avg_rating_no_reviews() {
        let p = make_proposal("1", "Test", "submitted", "presentation_40");
        assert!(avg_rating(&p).abs() < f64::EPSILON);
    }

    #[test]
    fn avg_rating_single_review() {
        let p = make_proposal_with_reviews("1", "Test", &[(4.0, 3.0, 5.0)]);
        assert!((avg_rating(&p) - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn avg_rating_multiple_reviews() {
        let p = make_proposal_with_reviews("1", "Test", &[(3.0, 3.0, 3.0), (5.0, 5.0, 5.0)]);
        assert!((avg_rating(&p) - 12.0).abs() < f64::EPSILON);
    }
}
