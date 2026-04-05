use anyhow::Result;
use colored::Colorize;
use console::Key;
use dialoguer::FuzzySelect;

use crate::client::TrpcClient;
use crate::types::Proposal;
use crate::{config, display, ui};

use super::display::{TABLE_HEADER, filter_summary, format_item};
use super::filters::{Filters, apply_filters};
use super::review::prompt_and_submit_review;

pub async fn list_interactive(client: &TrpcClient, all_proposals: &[Proposal]) -> Result<()> {
    if all_proposals.is_empty() {
        println!("No proposals found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut filters = Filters::default();
    let mut cursor = 0usize;

    loop {
        let filtered = apply_filters(all_proposals, &filters);
        let summary = filter_summary(&filters);

        if filtered.is_empty() {
            println!("No proposals match current filters. Press enter to adjust filters.");
            super::interactive::show_filter_menu(&mut filters)?;
            continue;
        }

        let menu_label = format!("⚙ Filter & Sort  ({summary})");
        let mut items: Vec<String> = vec![menu_label];
        items.extend(filtered.iter().map(|p| format_item(p)));

        let default = (cursor + 1).min(items.len() - 1);

        // Cap list height so the header/prompt stays visible, accounting for wrapping
        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{}/{} proposals\n  {TABLE_HEADER}\n  {hints}",
                filtered.len(),
                all_proposals.len()
            ))
            .items(&items)
            .default(default)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(0) => {
                show_filter_menu(&mut filters)?;
                cursor = 0;
            }
            Some(idx) => {
                cursor = idx - 1;
                let proposal_ids: Vec<&str> = filtered.iter().map(|p| p.id.as_str()).collect();
                cursor = show_detail_loop(client, &proposal_ids, cursor).await?;
            }
            None => break,
        }
    }

    Ok(())
}

pub fn show_filter_menu(filters: &mut Filters) -> Result<()> {
    use dialoguer::{MultiSelect, Select};

    use super::filters::{FORMATS, SORT_FIELDS, SORT_LABELS, STATUSES};

    let term = console::Term::stderr();
    term.clear_screen()?;

    // Status filter
    let status_defaults: Vec<bool> = STATUSES
        .iter()
        .map(|s| filters.statuses.contains(s))
        .collect();
    let status_labels: Vec<String> = STATUSES
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    println!(
        "{}",
        "Filter by status (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&status_labels)
        .defaults(&status_defaults)
        .interact()?;
    filters.statuses = selected.iter().map(|&i| STATUSES[i]).collect();

    // Format filter
    let format_defaults: Vec<bool> = FORMATS
        .iter()
        .map(|f| {
            if filters.formats.is_empty() {
                true
            } else {
                filters.formats.contains(f)
            }
        })
        .collect();
    let format_labels: Vec<&str> = FORMATS.iter().map(|f| f.label()).collect();

    println!(
        "\n{}",
        "Filter by format (space to toggle, enter to confirm):".bold()
    );
    let selected = MultiSelect::new()
        .items(&format_labels)
        .defaults(&format_defaults)
        .interact()?;
    filters.formats = if selected.len() == FORMATS.len() {
        vec![]
    } else {
        selected.iter().map(|&i| FORMATS[i]).collect()
    };

    // Sort field
    let sort_default = SORT_FIELDS
        .iter()
        .position(|&s| s == filters.sort_by)
        .unwrap_or(0);

    println!("\n{}", "Sort by:".bold());
    let sort_idx = Select::new()
        .items(SORT_LABELS)
        .default(sort_default)
        .interact()?;
    filters.sort_by = SORT_FIELDS[sort_idx];

    // Sort direction
    let dir_default = usize::from(filters.sort_asc);
    println!("\n{}", "Sort direction:".bold());
    let dir_idx = Select::new()
        .items(["Descending ↓", "Ascending ↑"])
        .default(dir_default)
        .interact()?;
    filters.sort_asc = dir_idx == 1;

    term.clear_screen()?;
    Ok(())
}

async fn show_detail_loop(
    client: &TrpcClient,
    proposal_ids: &[&str],
    start: usize,
) -> Result<usize> {
    let reviewer_name = config::load().ok().and_then(|c| c.name);
    let mut idx = start;
    let total = proposal_ids.len();

    loop {
        let sp = ui::spinner("Loading…");
        let proposal = super::fetch_one(client, proposal_ids[idx]).await?;
        sp.finish_and_clear();

        let content = display::render_proposal_detail(&proposal);

        // Build nav hints — scroll hints added dynamically by the pager
        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        // Use the longest possible hint string for viewport sizing
        let mut nav_full = nav.clone();
        nav_full.extend(["↑↓/jk scroll", "^u/^d half-page", "r review", "q/esc back"]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        // Build the actual footer shown to the user
        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
        nav.push("r review");
        nav.push("q/esc back");
        let footer = nav.join(" · ").dimmed().to_string();

        loop {
            let header = if pager.is_scrollable() {
                format!(
                    "[{}/{}] ↕ {}/{}",
                    idx + 1,
                    total,
                    pager.scroll_offset() + 1,
                    pager.line_count()
                )
            } else {
                format!("[{}/{}]", idx + 1, total)
            };

            pager.render(&header.dimmed().to_string(), &footer)?;

            match pager.handle_key()? {
                ui::pager::Action::Redraw => {}
                ui::pager::Action::Custom(key) => match key {
                    Key::ArrowLeft | Key::Char('h') => {
                        idx = idx.saturating_sub(1);
                        break;
                    }
                    Key::ArrowRight | Key::Char('l') => {
                        if idx + 1 < total {
                            idx += 1;
                        }
                        break;
                    }
                    Key::Char('r') => {
                        println!();
                        prompt_and_submit_review(client, &proposal, reviewer_name.as_deref())
                            .await?;
                        break;
                    }
                    Key::Escape | Key::Char('q') => {
                        pager.clear()?;
                        return Ok(idx);
                    }
                    _ => {}
                },
            }
        }
    }
}
