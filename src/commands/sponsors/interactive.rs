use anyhow::Result;
use colored::Colorize;
use console::Key;
use dialoguer::FuzzySelect;

use crate::client::TrpcClient;
use crate::display;
use crate::types::SponsorForConference;
use crate::ui;

pub fn list_interactive(_client: &TrpcClient, sponsors: &[SponsorForConference]) -> Result<()> {
    if sponsors.is_empty() {
        println!("No sponsors found.");
        return Ok(());
    }

    let hints = "↑↓ navigate · type to search · enter select · esc quit".dimmed();
    let mut cursor = 0usize;

    loop {
        let mut items: Vec<String> = Vec::with_capacity(sponsors.len());
        items.extend(sponsors.iter().map(display::format_sponsor_row));

        let default = cursor.min(items.len().saturating_sub(1));

        // Cap list height so the header/prompt stays visible, accounting for wrapping
        let max_rows = ui::max_visible_items(&items, 4);

        let selection = FuzzySelect::new()
            .with_prompt(format!(
                "{} sponsors\n  {}\n  {hints}",
                sponsors.len(),
                display::SPONSOR_TABLE_HEADER,
            ))
            .items(&items)
            .default(default)
            .max_length(max_rows)
            .highlight_matches(false)
            .interact_opt()?;

        match selection {
            Some(idx) => {
                cursor = idx;
                let ids: Vec<&str> = sponsors.iter().map(|s| s.id.as_str()).collect();
                cursor = show_detail_loop(sponsors, &ids, cursor)?;
            }
            None => break,
        }
    }

    Ok(())
}

fn show_detail_loop(
    sponsors: &[SponsorForConference],
    ids: &[&str],
    start: usize,
) -> Result<usize> {
    let mut idx = start;
    let total = ids.len();

    loop {
        let sponsor = &sponsors[idx];
        let content = display::render_sponsor_detail(sponsor);

        let mut nav = vec![];
        if idx > 0 {
            nav.push("← prev");
        }
        if idx + 1 < total {
            nav.push("→ next");
        }
        let mut nav_full = nav.clone();
        nav_full.extend(["↑↓/jk scroll", "^u/^d half-page", "q/esc back"]);
        let footer_measure = nav_full.join(" · ");

        let mut pager = ui::Pager::new(&content, &footer_measure);

        if pager.is_scrollable() {
            nav.push("↑↓/jk scroll");
            nav.push("^u/^d half-page");
        }
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
