use anyhow::{Context, Result};
use colored::Colorize;
use console::{Key, Term};
use dialoguer::{FuzzySelect, MultiSelect, Select};
use terminal_size::{Width, terminal_size};

use crate::client::TrpcClient;
use crate::config;
use crate::display;
use crate::types::{Proposal, ReviewScore};

fn term_width() -> usize {
    terminal_size().map_or(100, |(Width(w), _)| w as usize)
}

fn truncate(s: &str, max: usize) -> String {
    let char_count: usize = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

// --- Filter & sort state ---

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
        _ => {
            // "created" — by _createdAt
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
    let result = total / count as f64;
    result
}

fn format_item(p: &Proposal) -> String {
    let speakers: Vec<&str> = p.speakers.iter().map(|s| s.name.as_str()).collect();
    let speaker_str = speakers.join(", ");
    let format = display::humanize_format(p.format.as_deref().unwrap_or("-"));
    let status = display::pad_and_colorize_status(&p.status, 12);

    let prefix_len = 12 + 1 + 16 + 1;
    let prefix = format!("{status} {format:<16} ");
    let remaining = term_width().saturating_sub(prefix_len + 4);
    let title_budget = remaining * 2 / 3;
    let speaker_budget = remaining.saturating_sub(title_budget + 3);

    let title = truncate(&p.title, title_budget);
    if speaker_str.is_empty() {
        format!("{prefix}{title}")
    } else {
        let speaker = truncate(&speaker_str, speaker_budget);
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
        vec![] // all selected = no filter
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

pub async fn list() -> Result<()> {
    let cfg = config::load().context("Not logged in. Run `cnctl login` first.")?;
    let client = TrpcClient::from_config(&cfg);
    list_with(&client).await
}

pub async fn get(id: &str) -> Result<()> {
    let cfg = config::load().context("Not logged in. Run `cnctl login` first.")?;
    let client = TrpcClient::from_config(&cfg);
    get_with(&client, id).await
}

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<Proposal>> {
    client.query("proposal.admin.list", None).await
}

pub async fn list_with(client: &TrpcClient) -> Result<()> {
    let all_proposals = fetch_all(client).await?;

    if all_proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let header = format!("{:<12} {:<16} {}", "STATUS", "FORMAT", "TITLE · SPEAKER");
    let mut filters = Filters::default();
    let mut cursor = 0usize;

    loop {
        let filtered = apply_filters(&all_proposals, &filters);
        let summary = filter_summary(&filters);

        if filtered.is_empty() {
            println!("No proposals match current filters. Press enter to adjust filters.");
            show_filter_menu(&mut filters)?;
            continue;
        }

        let menu_label = format!("⚙ Filter & Sort  ({summary})");
        let mut items: Vec<String> = vec![menu_label];
        items.extend(filtered.iter().map(|p| format_item(p)));

        // Clamp cursor to valid range (offset by 1 for filter menu item)
        let default = (cursor + 1).min(items.len() - 1);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{}/{} proposals — type to search, esc to quit\n  {header}",
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
                cursor = idx - 1; // offset for filter menu item
                let proposal_ids: Vec<&str> = filtered.iter().map(|p| p.id.as_str()).collect();
                cursor = show_detail_loop(client, &proposal_ids, cursor).await?;
            }
            None => break,
        }
    }

    Ok(())
}

async fn show_detail_loop(
    client: &TrpcClient,
    proposal_ids: &[&str],
    start: usize,
) -> Result<usize> {
    let term = Term::stderr();
    let mut idx = start;
    let total = proposal_ids.len();

    loop {
        term.clear_screen()?;
        let pos = format!("[{}/{}]", idx + 1, total);
        println!("{}", pos.dimmed());
        get_with(client, proposal_ids[idx]).await?;

        let nav = "  ← prev · → next · q back to list";
        println!("\n{}", nav.dimmed());

        match term.read_key()? {
            Key::ArrowLeft | Key::Char('h' | 'k') => {
                idx = idx.saturating_sub(1);
            }
            Key::ArrowRight | Key::Char('l' | 'j') => {
                if idx + 1 < total {
                    idx += 1;
                }
            }
            _ => {
                term.clear_screen()?;
                break;
            }
        }
    }

    Ok(idx)
}

pub async fn get_with(client: &TrpcClient, id: &str) -> Result<()> {
    let input = serde_json::json!({ "id": id });
    let proposal: Proposal = client.query("proposal.admin.getById", Some(&input)).await?;
    display::print_proposal_detail(&proposal);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii_within_limit() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_at_limit() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_ascii_over_limit() {
        assert_eq!(truncate("hello world", 5), "hell…");
    }

    #[test]
    fn truncate_multibyte_no_panic() {
        // "Andrés" has é (2 bytes) — slicing by byte index would panic
        assert_eq!(truncate("Andrés Valero", 5), "Andr…");
    }

    #[test]
    fn truncate_multibyte_within_limit() {
        assert_eq!(truncate("Andrés", 10), "Andrés");
    }

    #[test]
    fn truncate_multibyte_at_boundary() {
        // 6 chars: A n d r é s — should fit exactly
        assert_eq!(truncate("Andrés", 6), "Andrés");
    }

    #[test]
    fn truncate_multibyte_just_over() {
        // 6 chars truncated to 5: keep 4 + …
        assert_eq!(truncate("Andrés", 5), "Andr…");
    }

    #[test]
    fn truncate_emoji() {
        // 🇳🇴 is two chars (regional indicators), each 4 bytes
        assert_eq!(truncate("🇳🇴 Norway", 3), "🇳🇴…");
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_zero_max() {
        // saturating_sub(1) on 0 = 0, so take 0 chars + …
        assert_eq!(truncate("hello", 0), "…");
    }

    #[test]
    fn truncate_max_one() {
        // saturating_sub(1) on 1 = 0, take 0 chars + …
        assert_eq!(truncate("hello", 1), "…");
    }

    #[test]
    fn truncate_norwegian_chars() {
        assert_eq!(truncate("Friheten i Koden: Ære", 15), "Friheten i Kod…");
    }

    // -- Filter & sort tests --

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
        // (9 + 15) / 2 = 12.0
        assert!((avg_rating(&p) - 12.0).abs() < f64::EPSILON);
    }
}
