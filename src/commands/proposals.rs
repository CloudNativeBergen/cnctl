use anyhow::{Context, Result};
use console::Term;
use dialoguer::Select;
use terminal_size::{Width, terminal_size};

use crate::client::TrpcClient;
use crate::config;
use crate::display;
use crate::types::Proposal;

fn term_width() -> usize {
    terminal_size().map_or(100, |(Width(w), _)| w as usize)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
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
    let proposals = fetch_all(client).await?;

    if proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let items: Vec<String> = proposals
        .iter()
        .map(|p| {
            let speakers: Vec<&str> = p.speakers.iter().map(|s| s.name.as_str()).collect();
            let speaker_str = speakers.join(", ");
            let format = display::humanize_format(p.format.as_deref().unwrap_or("-"));
            let status = display::pad_and_colorize_status(&p.status, 12);

            // Fixed-width columns: status(12) + format(16) = 28 + padding
            let prefix_len = 12 + 1 + 16 + 1; // visual width (status + space + format + space)
            let prefix = format!("{status} {format:<16} ");
            let remaining = term_width().saturating_sub(prefix_len + 4); // 4 for selector chrome
            let title_budget = remaining * 2 / 3;
            let speaker_budget = remaining.saturating_sub(title_budget + 3); // 3 for " · "

            let title = truncate(&p.title, title_budget);
            if speaker_str.is_empty() {
                format!("{prefix}{title}")
            } else {
                let speaker = truncate(&speaker_str, speaker_budget);
                format!("{prefix}{title} · {speaker}")
            }
        })
        .collect();

    let header = format!("{:<12} {:<16} {}", "STATUS", "FORMAT", "TITLE · SPEAKER");

    loop {
        let selection = Select::new()
            .with_prompt(format!(
                "{} proposals — ↑↓ navigate, enter to view, esc to quit\n  {header}",
                proposals.len()
            ))
            .items(&items)
            .default(0)
            .max_length(20)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                let term = Term::stderr();
                term.clear_screen()?;
                get_with(client, &proposals[idx].id).await?;
                println!("\n  Press any key to return to list…");
                term.read_key()?;
                term.clear_screen()?;
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
