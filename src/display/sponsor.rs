use std::fmt::Write;

use colored::Colorize;

use crate::types::{SponsorForConference, SponsorStatus};

pub fn print_sponsor_list(sponsors: &[SponsorForConference]) {
    if sponsors.is_empty() {
        println!("No sponsors found.");
        return;
    }

    println!("{SPONSOR_TABLE_HEADER}");

    for s in sponsors {
        println!("{}", format_sponsor_row(s));
    }

    println!("\n{} sponsors", sponsors.len());
}

pub const SPONSOR_TABLE_HEADER: &str =
    "SPONSOR              STATUS         CONTRACT          TIER";

pub fn format_sponsor_row(s: &SponsorForConference) -> String {
    let name = s.sponsor.as_ref().map_or("Unknown", |sp| sp.name.as_str());
    let tier = s.tier.as_ref().map_or("-", |t| t.title.as_str());
    let contract = s.contract_status.as_deref().unwrap_or("-");

    // Pad the status text *before* colorizing so ANSI codes don't break alignment
    let status_padded = format!("{:<14}", s.status);
    let status_colored = colorize_status_str(&status_padded, s.status);

    format!(
        "{:<20} {} {:<17} {}",
        truncate(name, 18),
        status_colored,
        contract,
        tier
    )
}

/// Render sponsor details into a `String` (for scrollable views, etc.).
pub fn render_sponsor_detail(sponsor: &SponsorForConference) -> String {
    let mut buf = String::new();
    let name = sponsor
        .sponsor
        .as_ref()
        .map_or("Unknown", |s| s.name.as_str());

    writeln!(buf, "{}", name.bold()).unwrap();
    writeln!(buf, "ID:              {}", sponsor.id).unwrap();
    writeln!(buf, "Status:          {}", colorize_status(sponsor.status)).unwrap();
    if let Some(contract) = &sponsor.contract_status {
        writeln!(buf, "Contract:        {contract}").unwrap();
    }
    if let Some(invoice) = &sponsor.invoice_status {
        writeln!(buf, "Invoice:         {invoice}").unwrap();
    }
    if let Some(tier) = &sponsor.tier {
        writeln!(buf, "Tier:            {}", tier.title).unwrap();
    }
    if let Some(assigned) = &sponsor.assigned_to {
        writeln!(buf, "Assigned to:     {}", assigned.name).unwrap();
    }
    if let Some(value) = sponsor.contract_value {
        let currency = sponsor.contract_currency.as_deref().unwrap_or("NOK");
        writeln!(buf, "Contract value:  {value} {currency}").unwrap();
    }
    if let Some(website) = sponsor.sponsor.as_ref().and_then(|s| s.website.as_deref()) {
        writeln!(buf, "Website:         {website}").unwrap();
    }

    if !sponsor.contact_persons.is_empty() {
        writeln!(buf, "\nContacts:").unwrap();
        for c in &sponsor.contact_persons {
            let role = c.role.as_deref().unwrap_or("");
            let email = c.email.as_deref().unwrap_or("");
            let primary = if c.is_primary.unwrap_or(false) {
                " [primary]"
            } else {
                ""
            };
            writeln!(buf, "  - {} <{}> {}{}", c.name, email, role, primary).unwrap();
        }
    }

    if let Some(billing) = &sponsor.billing
        && (billing.email.is_some() || billing.reference.is_some())
    {
        writeln!(buf, "\nBilling:").unwrap();
        if let Some(email) = &billing.email {
            writeln!(buf, "  Email:     {email}").unwrap();
        }
        if let Some(reference) = &billing.reference {
            writeln!(buf, "  Reference: {reference}").unwrap();
        }
    }

    if let Some(notes) = &sponsor.notes
        && !notes.is_empty()
    {
        writeln!(buf, "\nNotes:\n{notes}").unwrap();
    }

    if !sponsor.tags.is_empty() {
        writeln!(buf, "\nTags: {}", sponsor.tags.join(", ")).unwrap();
    }

    buf
}

/// Print sponsor details to stdout.
pub fn print_sponsor_detail(sponsor: &SponsorForConference) {
    print!("{}", render_sponsor_detail(sponsor));
}

fn colorize_status(status: SponsorStatus) -> String {
    colorize_status_str(&status.to_string(), status)
}

fn colorize_status_str(label: &str, status: SponsorStatus) -> String {
    match status {
        SponsorStatus::ClosedWon => label.green().to_string(),
        SponsorStatus::Negotiating => label.yellow().to_string(),
        SponsorStatus::Contacted => label.cyan().to_string(),
        SponsorStatus::Prospect | SponsorStatus::Unknown => label.dimmed().to_string(),
        SponsorStatus::ClosedLost => label.red().to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    crate::ui::truncate(s, max)
}
