mod args;
pub use args::*;

use anyhow::Result;
use colored::Colorize;

use super::require_client;
use crate::client::TrpcClient;
use crate::types::{ProposalStatus, Speaker, SpeakerSummary};

pub async fn run(cmd: SpeakerCommand) -> Result<()> {
    match cmd {
        SpeakerCommand::List(args) => list(args).await,
        SpeakerCommand::Get { id, json } => get(&id, json).await,
        SpeakerCommand::Add(create_args) => create(create_args).await,
        SpeakerCommand::Delete { id, yes } => delete(&id, yes).await,
        SpeakerCommand::Broadcast {
            subject,
            message,
            sync,
        } => broadcast(subject.as_deref(), message.as_deref(), sync).await,
    }
}

pub async fn fetch_all(client: &TrpcClient) -> Result<Vec<SpeakerSummary>> {
    client.query("speaker.admin.list", None).await
}

pub async fn fetch_search(client: &TrpcClient, args: &ListArgs) -> Result<Vec<Speaker>> {
    client
        .query("speaker.admin.search", Some(&serde_json::to_value(args)?))
        .await
}

pub async fn fetch_one(client: &TrpcClient, id: &str) -> Result<Speaker> {
    client
        .query(
            "speaker.admin.getById",
            Some(&serde_json::json!({ "id": id })),
        )
        .await
}

async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    let statuses = args
        .status
        .clone()
        .unwrap_or_else(|| vec![ProposalStatus::Accepted, ProposalStatus::Confirmed]);

    // We fetch proposals to get the speakers with the desired status
    let proposal_args = crate::commands::proposals::ListArgs {
        status: Some(statuses),
        search: args.query.clone(),
        sort_by: args.sort,
        sort_order: args.order,
        ..Default::default()
    };

    let proposals = crate::commands::proposals::fetch_all(&client, &proposal_args).await?;

    // Extract unique speakers from proposals
    let mut speakers: Vec<SpeakerSummary> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for p in proposals {
        for s in p.speakers {
            if seen_ids.insert(s.id.clone()) {
                speakers.push(SpeakerSummary {
                    id: s.id,
                    name: s.name,
                    email: s.email,
                    title: None, // Summary from proposals might not have full title
                    slug: None,
                    image: s.image,
                });
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&speakers)?);
    } else {
        if speakers.is_empty() {
            println!("No speakers found with the given filters.");
            return Ok(());
        }

        println!(
            "{}",
            "ID                   NAME                 EMAIL"
                .bold()
                .cyan()
        );
        for s in speakers {
            println!(
                "{:<20} {:<20} {}",
                s.id,
                s.name,
                s.email.as_deref().unwrap_or_default()
            );
        }
        println!(
            "\n{}",
            "Hint: Use `cnctl admin speakers get <ID>` for full bio and links.".dimmed()
        );
    }
    Ok(())
}

async fn get(id: &str, json: bool) -> Result<()> {
    let client = require_client()?;
    let speaker = fetch_one(&client, id).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&speaker)?);
    } else {
        println!("{} ({})", speaker.name.bold(), speaker.id.dimmed());
        if let Some(ref email) = speaker.email {
            println!("Email:   {email}");
        }
        if let Some(ref company) = speaker.company {
            println!("Company: {company}");
        }
        if let Some(ref title) = speaker.title {
            println!("Title:   {title}");
        }

        if !speaker.flags.is_empty() {
            println!("Flags:   {:?}", speaker.flags);
        }

        if !speaker.links.is_empty() {
            println!("\nLinks:");
            for link in speaker.links {
                println!("  - {link}");
            }
        }

        if !speaker.bio.is_empty() {
            println!("\nBio:");
            println!("{}", crate::types::portable_text_to_plain(&speaker.bio));
        }
    }
    Ok(())
}

async fn create(args: CreateArgs) -> Result<()> {
    let client = require_client()?;
    let speaker: Speaker = client
        .mutate("speaker.admin.create", &serde_json::to_value(args)?)
        .await?;
    println!(
        "Successfully created speaker {} (ID: {})",
        speaker.name, speaker.id
    );
    Ok(())
}

async fn delete(id: &str, yes: bool) -> Result<()> {
    if !yes && console::Term::stdout().is_term() {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!("Are you sure you want to delete speaker {id}?"))
            .default(false)
            .interact()?;

        if !confirmed {
            anyhow::bail!("Deletion cancelled.");
        }
    }

    let client = require_client()?;
    client
        .mutate::<serde_json::Value>("speaker.admin.delete", &serde_json::json!({ "id": id }))
        .await?;
    println!("Successfully deleted speaker {id}.");
    Ok(())
}

async fn broadcast(subject: Option<&str>, message: Option<&str>, sync: bool) -> Result<()> {
    let client = require_client()?;

    if sync {
        println!("Syncing speaker list with newsletter audience...");
        let res: serde_json::Value = client
            .mutate("speaker.admin.syncAudience", &serde_json::json!({}))
            .await?;
        println!("Sync response: {res:?}");
    }

    if let (Some(subject), Some(message)) = (subject, message) {
        // Wrap plain text in a basic Portable Text block
        let portable_text = serde_json::json!([{
            "_type": "block",
            "children": [{
                "_type": "span",
                "text": message
            }],
            "style": "normal"
        }]);

        client
            .mutate::<serde_json::Value>(
                "speaker.admin.broadcastEmail",
                &serde_json::json!({
                    "subject": subject,
                    "message": portable_text.to_string()
                }),
            )
            .await?;

        println!("Broadcast email sent successfully.");
    }

    Ok(())
}
