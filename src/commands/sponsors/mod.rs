mod args;
pub mod email;
mod interactive;

pub use args::{CreateArgs, EmailArgs, ListArgs, NoteArgs};

use anyhow::{Context, Result};

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

pub async fn fetch_activities(
    client: &TrpcClient,
    id: &str,
) -> Result<Vec<crate::types::SponsorActivity>> {
    let activities: Vec<crate::types::SponsorActivity> = client
        .query(
            "sponsor.crm.activities.list",
            Some(&serde_json::json!({ "sponsorForConferenceId": id })),
        )
        .await?;
    Ok(activities)
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

    let mut sponsor = sponsors
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Sponsor not found: {id}"))?;

    if let Ok(activities) = fetch_activities(&client, id).await {
        sponsor.activities = activities;
    }

    display::print_sponsor_detail(&sponsor);
    Ok(())
}

pub async fn history(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let sponsors = fetch_all(&client).await?;

    let mut sponsor = sponsors
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Sponsor not found: {id}"))?;

    if let Ok(activities) = fetch_activities(&client, id).await {
        sponsor.activities = activities;
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&sponsor.activities)?);
    } else {
        display::print_sponsor_history(&sponsor);
    }
    Ok(())
}

pub async fn add_note(args: NoteArgs) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.activities.create",
            &serde_json::json!({
                "sponsorForConferenceId": args.id,
                "activityType": args.kind,
                "description": args.description,
            }),
        )
        .await?;

    println!("Activity logged successfully.");
    Ok(())
}

pub async fn create(args: CreateArgs) -> Result<()> {
    let client = require_client()?;
    let config = crate::config::load()?;

    // 1. Create the base sponsor
    let sponsor: serde_json::Value = client
        .mutate(
            "sponsor.create",
            &serde_json::json!({
                "name": args.name,
                "website": args.website,
            }),
        )
        .await?;

    let sponsor_id = sponsor
        .get("id")
        .or_else(|| sponsor.get("_id"))
        .and_then(|id| id.as_str())
        .context(format!("Missing sponsor ID in response: {sponsor:?}"))?;

    // 2. Link to conference (CRM)
    let mut contact_persons = serde_json::json!([]);
    if let Some(name) = args.contact_name {
        contact_persons = serde_json::json!([{
            "_key": uuid::Uuid::new_v4().to_string(),
            "name": name,
            "email": args.contact_email,
            "isPrimary": true,
        }]);
    }

    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.create",
            &serde_json::json!({
                "sponsor": sponsor_id,
                "conference": config.conference_id,
                "status": args.status,
                "contractStatus": "none",
                "invoiceStatus": "not-sent",
                "notes": args.notes,
                "contactPersons": contact_persons,
            }),
        )
        .await?;

    println!("Sponsor '{}' added to CRM as {}.", args.name, args.status);
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
