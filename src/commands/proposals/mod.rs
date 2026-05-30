mod args;
mod display;
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

pub async fn fetch_all(client: &TrpcClient, args: &ListArgs) -> Result<Vec<Proposal>> {
    let mut payload = args.clone();
    if payload.unreviewed {
        payload.review_status = Some(crate::types::ReviewStatus::Unreviewed);
    }
    client
        .query(
            "proposal.admin.list",
            Some(&serde_json::to_value(&payload)?),
        )
        .await
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
    let all = fetch_all(&client, &args).await?;
    sp.finish_and_clear();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&all)?);
        Ok(())
    } else if args.has_cli_filters() || !console::Term::stdout().is_term() {
        if all.is_empty() {
            println!("No proposals match the given filters.");
            return Ok(());
        }

        println!("{}", display::TABLE_HEADER);
        for p in &all {
            println!("{}", display::format_item(p));
        }
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

pub async fn next_review() -> Result<()> {
    let client = require_client()?;
    let reviewer_name = crate::config::load().ok().and_then(|c| c.name);

    let sp = ui::spinner("Fetching next unreviewed proposal…");
    let input = serde_json::json!({});
    let proposal_opt: Option<Proposal> = client
        .query("proposal.admin.nextUnreviewed", Some(&input))
        .await?;
    sp.finish_and_clear();

    match proposal_opt {
        Some(proposal) => {
            crate::display::print_proposal_detail(&proposal);
            println!();
            review::prompt_and_submit_review(&client, &proposal, reviewer_name.as_deref()).await?;
        }
        None => {
            println!("No unreviewed proposals found. Great job!");
        }
    }

    Ok(())
}
