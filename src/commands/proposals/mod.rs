mod args;
mod display;
mod filters;
mod interactive;
mod review;

#[cfg(test)]
mod tests;

pub use args::{ListArgs, ReviewArgs};

use anyhow::Result;

use super::require_client;
use crate::client::TrpcClient;
use crate::types::{Proposal, ReviewInput};
use crate::ui;

// ── API helpers ──────────────────────────────────────────────────────────────

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

// ── Command entry points ─────────────────────────────────────────────────────

pub async fn list(args: ListArgs) -> Result<()> {
    let client = require_client()?;

    let sp = ui::spinner("Fetching proposals…");
    let all = fetch_all(&client).await?;
    sp.finish_and_clear();

    if args.json {
        list_json(&all, &args)
    } else if args.has_cli_filters() || !console::Term::stdout().is_term() {
        list_plain(&all, &args);
        Ok(())
    } else {
        interactive::list_interactive(&client, &all).await
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
        crate::display::print_proposal_detail(&proposal);
    }
    Ok(())
}

pub async fn review(args: ReviewArgs) -> Result<()> {
    use crate::types::ReviewScore;
    use crate::{config, display};

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
        review::prompt_and_submit_review(&client, &proposal, reviewer_name.as_deref()).await?;
    }

    Ok(())
}

// ── Output modes ─────────────────────────────────────────────────────────────

fn list_json(all: &[Proposal], args: &ListArgs) -> Result<()> {
    let filters = filters::Filters::from(args);
    let filtered = filters::apply_filters(all, &filters);
    println!("{}", serde_json::to_string_pretty(&filtered)?);
    Ok(())
}

fn list_plain(all: &[Proposal], args: &ListArgs) {
    let filters = filters::Filters::from(args);
    let filtered = filters::apply_filters(all, &filters);

    if filtered.is_empty() {
        println!("No proposals match the given filters.");
        return;
    }

    println!("{}", display::TABLE_HEADER);
    for p in &filtered {
        println!("{}", display::format_item(p));
    }
}
