mod proposal;
mod sponsor;
mod status;

pub use proposal::{pad_and_colorize_status, print_proposal_detail, render_proposal_detail};
pub use sponsor::{
    SPONSOR_TABLE_HEADER, format_sponsor_row, print_sponsor_detail, print_sponsor_history,
    print_sponsor_list, render_sponsor_detail,
};
pub use status::print_status;

pub fn print_agent_list<T: serde::Serialize>(
    data: T,
    total: usize,
    returned: usize,
) -> anyhow::Result<()> {
    let out = serde_json::json!({
        "data": data,
        "_meta": {
            "total": total,
            "returned": returned,
            "truncated": returned < total,
            "hint": "Use --limit or other filters to narrow results"
        }
    });
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
