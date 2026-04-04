use colored::Colorize;

use crate::types::Proposal;

pub fn print_proposal_detail(proposal: &Proposal) {
    println!("{}", proposal.title.bold());
    println!("ID:       {}", proposal.id);
    println!("Status:   {}", colorize_status(&proposal.status));
    if let Some(format) = &proposal.format {
        println!("Format:   {format}");
    }
    if let Some(level) = &proposal.level {
        println!("Level:    {level}");
    }
    if let Some(language) = &proposal.language {
        println!("Language: {language}");
    }

    if !proposal.speakers.is_empty() {
        println!("\nSpeakers:");
        for s in &proposal.speakers {
            let email = s.email.as_deref().unwrap_or("");
            println!("  - {} <{}>", s.name, email);
        }
    }

    if !proposal.topics.is_empty() {
        let topics: Vec<&str> = proposal
            .topics
            .iter()
            .filter_map(|t| t.title.as_deref())
            .collect();
        if !topics.is_empty() {
            println!("\nTopics: {}", topics.join(", "));
        }
    }

    if let Some(outline) = &proposal.outline
        && !outline.is_empty()
    {
        println!("\nOutline:\n{outline}");
    }

    if !proposal.reviews.is_empty() {
        println!("\nReviews:");
        for r in &proposal.reviews {
            let reviewer = r.reviewer.as_ref().map_or("Anonymous", |r| r.name.as_str());
            let score = r.score.as_ref().map_or_else(
                || "-".into(),
                |s| {
                    format!(
                        "{:.0}/30 (content:{:.0} relevance:{:.0} speaker:{:.0})",
                        s.total(),
                        s.content,
                        s.relevance,
                        s.speaker
                    )
                },
            );
            let comment = r.comment.as_deref().unwrap_or("");
            println!("  {reviewer} ({score}): {comment}");
        }
    }
}

fn colorize_status(status: &str) -> String {
    match status {
        "submitted" => status.yellow().to_string(),
        "accepted" => status.green().to_string(),
        "confirmed" => status.green().bold().to_string(),
        "rejected" => status.red().to_string(),
        "waitlisted" => status.cyan().to_string(),
        "withdrawn" | "draft" => status.dimmed().to_string(),
        _ => status.to_string(),
    }
}
