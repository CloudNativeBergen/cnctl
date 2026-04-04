use anyhow::{Context, Result};
use dialoguer::Select;

use crate::client::TrpcClient;
use crate::config;
use crate::display;
use crate::types::Proposal;

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
    let proposals = fetch_all(client).await?;

    if proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let items: Vec<String> = proposals
        .iter()
        .map(|p| {
            let speakers: Vec<&str> = p.speakers.iter().map(|s| s.name.as_str()).collect();
            let speaker_str = if speakers.is_empty() {
                String::new()
            } else {
                format!(" ({})", speakers.join(", "))
            };
            let format = p.format.as_deref().unwrap_or("-");
            format!("{:<12} {:<16} {}{}", p.status, format, p.title, speaker_str)
        })
        .collect();

    loop {
        let selection = Select::new()
            .with_prompt(format!(
                "{} proposals — ↑↓ to navigate, enter to view, q to quit",
                proposals.len()
            ))
            .items(&items)
            .default(0)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                println!();
                get_with(client, &proposals[idx].id).await?;
                println!();
            }
            None => break,
        }
    }

    Ok(())
}

pub async fn get_with(client: &TrpcClient, id: &str) -> Result<()> {
    let input = serde_json::json!({ "id": id });
    let proposal: Proposal = client.query("proposal.admin.getById", Some(&input)).await?;
    display::print_proposal_detail(&proposal);
    Ok(())
}
