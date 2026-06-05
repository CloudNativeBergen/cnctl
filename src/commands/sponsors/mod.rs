mod args;
pub mod email;
mod interactive;

pub use args::{CreateArgs, EmailArgs, ListArgs, NoteArgs, UpdateArgs, UpdateContactsArgs};

use anyhow::{Context, Result};

use super::require_client;
use crate::client::TrpcClient;
use crate::display;
use crate::types::SponsorForConference;

// ── API helpers ──────────────────────────────────────────────────────────────

pub async fn fetch_all(client: &TrpcClient, args: &ListArgs) -> Result<Vec<SponsorForConference>> {
    let sponsors: Vec<SponsorForConference> = client
        .query("sponsor.crm.list", Some(&serde_json::to_value(args)?))
        .await?;
    Ok(sponsors)
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<SponsorForConference> {
    let sponsor: SponsorForConference = client
        .query(
            "sponsor.crm.getById",
            Some(&serde_json::json!({ "id": id })),
        )
        .await?;
    Ok(sponsor)
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

    if args.json {
        let all = fetch_all(&client, &args).await?;
        println!("{}", serde_json::to_string_pretty(&all)?);
    } else if args.search.is_some()
        || args.status.is_some()
        || args.assigned_to.is_some()
        || args.unassigned
        || args.tags.is_some()
        || args.tiers.is_some()
        || args.sort_by.is_some()
        || args.sort_order.is_some()
        || args.stale_days.is_some()
        || args.due
        || args.has_follow_up
        || args.has_contact
        || !console::Term::stdout().is_term()
    {
        let all = fetch_all(&client, &args).await?;
        if all.is_empty() {
            println!("No sponsors match the given filters.");
        } else {
            println!("{}", display::SPONSOR_TABLE_HEADER);
            for s in &all {
                println!("{}", display::format_sponsor_row(s));
            }
            println!("\n{} sponsors", all.len());
        }
    } else {
        interactive::list_interactive(&client, args).await?;
    }
    Ok(())
}

pub async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let sponsor = fetch_one(&client, id).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&sponsor)?);
    } else {
        display::print_sponsor_detail(&sponsor);
    }
    Ok(())
}

pub async fn update(args: UpdateArgs) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": args.id,
                "nextFollowUpAt": args.next_follow_up,
                "linkedinUrl": args.linkedin_url,
                "notes": args.notes,
                "assignedTo": args.assigned_to,
            }),
        )
        .await?;

    println!("Sponsor {} updated successfully.", args.id);
    Ok(())
}

pub async fn update_contacts(args: UpdateContactsArgs) -> Result<()> {
    let client = require_client()?;
    let contact_persons = serde_json::json!([{
        "_key": uuid::Uuid::new_v4().to_string(),
        "name": args.name,
        "email": args.email,
        "isPrimary": true,
    }]);

    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": args.id,
                "contactPersons": contact_persons,
            }),
        )
        .await?;

    println!("Contacts for sponsor {} updated successfully.", args.id);
    Ok(())
}

pub async fn fetch_organizers(client: &TrpcClient) -> Result<Vec<crate::types::SpeakerRef>> {
    let organizers: Vec<crate::types::SpeakerRef> =
        client.query("sponsor.crm.listOrganizers", None).await?;
    Ok(organizers)
}

pub async fn assign(id: &str, speaker_id: Option<&str>) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.update",
            &serde_json::json!({
                "id": id,
                "assignedTo": speaker_id,
            }),
        )
        .await?;

    match speaker_id {
        Some(sid) => println!("Sponsor {id} assigned to speaker {sid}."),
        None => println!("Sponsor {id} unassigned."),
    }
    Ok(())
}

pub async fn history(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let mut sponsor = fetch_one(&client, id).await?;
    sponsor.activities = fetch_activities(&client, id).await?;

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

pub async fn move_stage(id: &str, stage: crate::types::SponsorStatus) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.moveStage",
            &serde_json::json!({
                "id": id,
                "newStatus": stage,
            }),
        )
        .await?;

    println!("Sponsor moved to stage {stage}.");
    Ok(())
}

pub async fn update_invoice(id: &str, status: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.updateInvoiceStatus",
            &serde_json::json!({
                "id": id,
                "status": status,
            }),
        )
        .await?;

    println!("Invoice status updated to {status}.");
    Ok(())
}

pub async fn update_contract(id: &str, status: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.updateContractStatus",
            &serde_json::json!({
                "id": id,
                "status": status,
            }),
        )
        .await?;

    println!("Contract status updated to {status}.");
    Ok(())
}

pub async fn send_contract(id: &str, template: Option<&str>) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.sendContract",
            &serde_json::json!({
                "id": id,
                "templateSlug": template,
            }),
        )
        .await?;

    println!("Contract generated and sent successfully.");
    Ok(())
}

pub async fn delete_activity(id: &str) -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>(
            "sponsor.crm.activities.delete",
            &serde_json::json!({ "id": id }),
        )
        .await?;

    println!("Activity deleted successfully.");
    Ok(())
}

pub async fn signature_status(id: &str) -> Result<()> {
    let client = require_client()?;
    let res: serde_json::Value = client
        .mutate(
            "sponsor.crm.checkSignatureStatus",
            &serde_json::json!({ "id": id }),
        )
        .await?;

    let status = res
        .get("contractStatus")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    println!("Signature status synced. Current contract status: {status}");
    Ok(())
}

pub async fn sync_audience() -> Result<()> {
    let client = require_client()?;
    client
        .mutate::<serde_json::Value>("sponsor.crm.syncAudience", &serde_json::json!({}))
        .await?;

    println!("Sponsor email audience synced successfully.");
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
