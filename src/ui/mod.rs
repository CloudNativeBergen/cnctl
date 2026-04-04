pub mod pager;

pub use pager::Pager;

use indicatif::{ProgressBar, ProgressStyle};
use terminal_size::{Width, terminal_size};

pub fn term_width() -> usize {
    terminal_size().map_or(100, |(Width(w), _)| w as usize)
}

pub fn truncate(s: &str, max: usize) -> String {
    let char_count: usize = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii_within_limit() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_at_limit() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_ascii_over_limit() {
        assert_eq!(truncate("hello world", 5), "hell…");
    }

    #[test]
    fn truncate_multibyte_no_panic() {
        assert_eq!(truncate("Andrés Valero", 5), "Andr…");
    }

    #[test]
    fn truncate_multibyte_within_limit() {
        assert_eq!(truncate("Andrés", 10), "Andrés");
    }

    #[test]
    fn truncate_multibyte_at_boundary() {
        assert_eq!(truncate("Andrés", 6), "Andrés");
    }

    #[test]
    fn truncate_multibyte_just_over() {
        assert_eq!(truncate("Andrés", 5), "Andr…");
    }

    #[test]
    fn truncate_emoji() {
        assert_eq!(truncate("🇳🇴 Norway", 3), "🇳🇴…");
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_zero_max() {
        assert_eq!(truncate("hello", 0), "…");
    }

    #[test]
    fn truncate_max_one() {
        assert_eq!(truncate("hello", 1), "…");
    }

    #[test]
    fn truncate_norwegian_chars() {
        assert_eq!(truncate("Friheten i Koden: Ære", 15), "Friheten i Kod…");
    }
}
