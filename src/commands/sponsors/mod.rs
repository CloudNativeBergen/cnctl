mod args;
pub mod email;
mod interactive;

pub use args::{EmailArgs, ListArgs};

use anyhow::Result;

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::{SponsorForConference, SponsorStatus};

// ── API helpers ──────────────────────────────────────────────────────────────

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<SponsorForConference>> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::json!({})))
        .await?;
    Ok(sponsors)
}

// ── Command entry points ─────────────────────────────────────────────────────

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
        interactive::list_interactive(&client, &all)?;
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

// ── Internal helpers ─────────────────────────────────────────────────────────

fn filter_by_status<'a>(
    sponsors: &'a [SponsorForConference],
    statuses: Option<&[SponsorStatus]>,
) -> Vec<&'a SponsorForConference> {
    match statuses {
        Some(s) if !s.is_empty() => sponsors
            .iter()
            .filter(|sp| s.contains(&sp.status))
            .collect(),
        _ => sponsors.iter().collect(),
    }
}
