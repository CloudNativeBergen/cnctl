use colored::Colorize;

use crate::types::SponsorForConference;

pub fn print_sponsor_list(sponsors: &[SponsorForConference]) {
    if sponsors.is_empty() {
        println!("No sponsors found.");
        return;
    }

    println!(
        "{:<40} {:<20} {:<14} {:<14} TIER",
        "ID", "SPONSOR", "STATUS", "CONTRACT"
    );
    println!("{}", "─".repeat(100));

    for s in sponsors {
        let name = s.sponsor.as_ref().map_or("Unknown", |sp| sp.name.as_str());
        let tier = s.tier.as_ref().map_or("-", |t| t.title.as_str());
        let contract = s.contract_status.as_deref().unwrap_or("-");

        println!(
            "{:<40} {:<20} {:<23} {:<14} {}",
            s.id,
            truncate(name, 18),
            colorize_status(&s.status),
            contract,
            tier
        );
    }

    println!("\n{} sponsors", sponsors.len());
}

pub fn print_sponsor_detail(sponsor: &SponsorForConference) {
    let name = sponsor
        .sponsor
        .as_ref()
        .map_or("Unknown", |s| s.name.as_str());

    println!("{}", name.bold());
    println!("ID:              {}", sponsor.id);
    println!("Status:          {}", colorize_status(&sponsor.status));
    if let Some(contract) = &sponsor.contract_status {
        println!("Contract:        {contract}");
    }
    if let Some(invoice) = &sponsor.invoice_status {
        println!("Invoice:         {invoice}");
    }
    if let Some(tier) = &sponsor.tier {
        println!("Tier:            {}", tier.title);
    }
    if let Some(assigned) = &sponsor.assigned_to {
        println!("Assigned to:     {}", assigned.name);
    }
    if let Some(value) = sponsor.contract_value {
        let currency = sponsor.contract_currency.as_deref().unwrap_or("NOK");
        println!("Contract value:  {value} {currency}");
    }
    if let Some(website) = sponsor.sponsor.as_ref().and_then(|s| s.website.as_deref()) {
        println!("Website:         {website}");
    }

    if !sponsor.contact_persons.is_empty() {
        println!("\nContacts:");
        for c in &sponsor.contact_persons {
            let role = c.role.as_deref().unwrap_or("");
            let email = c.email.as_deref().unwrap_or("");
            let primary = if c.is_primary.unwrap_or(false) {
                " [primary]"
            } else {
                ""
            };
            println!("  - {} <{}> {}{}", c.name, email, role, primary);
        }
    }

    if let Some(billing) = &sponsor.billing
        && (billing.email.is_some() || billing.reference.is_some())
    {
        println!("\nBilling:");
        if let Some(email) = &billing.email {
            println!("  Email:     {email}");
        }
        if let Some(reference) = &billing.reference {
            println!("  Reference: {reference}");
        }
    }

    if let Some(notes) = &sponsor.notes
        && !notes.is_empty()
    {
        println!("\nNotes:\n{notes}");
    }

    if !sponsor.tags.is_empty() {
        println!("\nTags: {}", sponsor.tags.join(", "));
    }
}

fn colorize_status(status: &str) -> String {
    match status {
        "closed-won" => status.green().to_string(),
        "negotiating" => status.yellow().to_string(),
        "contacted" => status.cyan().to_string(),
        "prospect" => status.dimmed().to_string(),
        "closed-lost" => status.red().to_string(),
        _ => status.to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}
