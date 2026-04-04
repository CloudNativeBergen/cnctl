use colored::Colorize;

use crate::types::Proposal;

pub fn print_proposal_detail(proposal: &Proposal) {
    println!("{}", proposal.title.bold());
    println!("ID:       {}", proposal.id);
    println!("Status:   {}", colorize_status(&proposal.status));
    if let Some(format) = &proposal.format {
        println!("Format:   {}", humanize_format(format));
    }
    if let Some(level) = &proposal.level {
        println!("Level:    {}", capitalize(level));
    }
    if let Some(language) = &proposal.language {
        println!("Language: {}", capitalize(language));
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
    let label = humanize_status(status);
    match status {
        "submitted" => label.yellow().to_string(),
        "accepted" => label.green().to_string(),
        "confirmed" => label.green().bold().to_string(),
        "rejected" => label.red().to_string(),
        "waitlisted" => label.cyan().to_string(),
        "withdrawn" | "draft" => label.dimmed().to_string(),
        _ => label.to_string(),
    }
}

pub fn humanize_format(format: &str) -> &str {
    match format {
        "lightning_10" => "Lightning 10min",
        "presentation_20" => "Talk 20min",
        "presentation_25" => "Talk 25min",
        "presentation_40" => "Talk 40min",
        "presentation_45" => "Talk 45min",
        "workshop_120" => "Workshop 2h",
        "workshop_240" => "Workshop 4h",
        other => other,
    }
}

pub fn pad_and_colorize_status(status: &str, width: usize) -> String {
    let label = humanize_status(status);
    let padded = format!("{label:<width$}");
    match status {
        "submitted" => padded.yellow().to_string(),
        "accepted" => padded.green().to_string(),
        "confirmed" => padded.green().bold().to_string(),
        "rejected" => padded.red().to_string(),
        "waitlisted" => padded.cyan().to_string(),
        "withdrawn" | "draft" => padded.dimmed().to_string(),
        _ => padded,
    }
}

pub fn humanize_status(status: &str) -> &str {
    match status {
        "submitted" => "Submitted",
        "accepted" => "Accepted",
        "confirmed" => "Confirmed",
        "rejected" => "Rejected",
        "waitlisted" => "Waitlisted",
        "withdrawn" => "Withdrawn",
        "draft" => "Draft",
        "deleted" => "Deleted",
        other => other,
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
