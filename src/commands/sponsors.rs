use anyhow::Result;
use clap::Args;
use colored::Colorize;
use console::Key;
use dialoguer::FuzzySelect;

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::{SponsorForConference, SponsorStatus};
use crate::ui;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by status (comma-separated)
    #[arg(long, value_delimiter = ',', value_enum)]
    pub status: Option<Vec<SponsorStatus>>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;
    let all = fetch_all(&client).await?;

    if args.json {
        let filtered = filter_by_status(&all, args.status.as_deref());
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else if args.status.is_some() || !console::Term::stdout().is_term() {
        let filtered = filter_by_status(&all, args.status.as_deref());
        if filtered.is_empty() {
            println!("No sponsors match the given filters.");
        } else {
            println!("{}", display::SPONSOR_TABLE_HEADER);
            for s in &filtered {
                println!("{}", display::format_sponsor_row(s));
            }
            println!("\n{} sponsors", filtered.len());
        }
    } else {
        list_interactive(&client, &all)?;
    }
    Ok(())
}

pub async fn get(id: &str) -> Result<()> {
    let client = require_client()?;
    let sponsors = fetch_all(&client).await?;

    let sponsor = sponsors
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Sponsor not found: {id}"))?;

    display::print_sponsor_detail(sponsor);
    Ok(())
}

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<SponsorForConference>> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::json!({})))
        .await?;
    Ok(sponsors)
}

fn filter_by_status<'a>(
    sponsors: &'a [SponsorForConference],
    statuses: Option<&[SponsorStatus]>,
) -> Vec<&'a SponsorForConference> {
    match statuses {
        Some(s) if !s.is_empty() => sponsors.iter().filter(|sp| s.contains(&sp.status)).collect(),
        _ => sponsors.iter().collect(),
    }
}

fn list_interactive(
    _client: &TrpcClient,
    sponsors: &[SponsorForConference],
) -> Result<()> {
    if sponsors.is_empty() {
        println!("No sponsors found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut cursor = 0usize;

    loop {
        let mut items: Vec<String> = Vec::with_capacity(sponsors.len());
        items.extend(sponsors.iter().map(display::format_sponsor_row));

        let default = cursor.min(items.len().saturating_sub(1));

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{} sponsors\n  {}\n  {hints}",
                sponsors.len(),
                display::SPONSOR_TABLE_HEADER,
            ))
            .items(&items)
            .default(default)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                cursor = idx;
                let ids: Vec<&str> = sponsors.iter().map(|s| s.id.as_str()).collect();
                cursor = show_detail_loop(sponsors, &ids, cursor)?;
            }
            None => break,
        }
    }

    Ok(())
}

fn show_detail_loop(
    sponsors: &[SponsorForConference],
    ids: &[&str],
    start: usize,
) -> Result<usize> {
    let mut idx = start;
    let total = ids.len();

    loop {
        let sponsor = &sponsors[idx];
        let content = display::render_sponsor_detail(sponsor);

        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        let mut nav_full = nav.clone();
        nav_full.extend(["↑↓/jk scroll", "^u/^d half-page", "q/esc back"]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
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
