use anyhow::Result;

use crate::config;

pub fn run() -> Result<()> {
    if config::delete()? {
        println!("Logged out.");
    } else {
        println!("Not logged in.");
    }
    Ok(())
}
